{-# LANGUAGE BangPatterns #-}

-- | Pure Monte Carlo risk simulation.
--
-- = Algebraic data types
--
-- Haskell models domain concepts with algebraic data types (ADTs).
-- 'RiskEvent' and 'SimulationResult' are product types, each field
-- is labelled with a record accessor that doubles as a function
-- (e.g. @probability :: RiskEvent -> Double@).  Unlike Rust structs,
-- these are immutable by default and live on the GC-managed heap.
--
-- = Purity and RNG threading
--
-- A pure function cannot hold mutable state, so we thread the random
-- number generator (StdGen) explicitly through every step that needs
-- a random draw.  Each call to 'randomR' returns a new generator
-- alongside the sampled value.  This is the same pattern Rust uses
-- with @&mut rng@, but made explicit in the type signature rather
-- than hidden behind a mutable reference.
--
-- = Laziness gotchas
--
-- Haskell is lazy by default, which means intermediate accumulators
-- like running totals can build up unevaluated thunks (deferred
-- computations) that eat memory.  We use BangPatterns (the @!@ prefix
-- on bindings) and 'foldl'' (strict left fold) to force evaluation at
-- each step, mirroring how Rust eagerly evaluates every statement.

module RiskSampler
  ( RiskEvent(..)
  , SimulationResult(..)
  , simulate
  ) where

import           Data.List   (sort)
import           System.Random (StdGen, mkStdGen, randomR)

-- | A single risk event with a firing probability and maximum loss.
--
-- This is a product type (analogous to a Rust struct).  Every field
-- accessor is automatically a top-level function, so @eventName e@
-- extracts the name from a RiskEvent value @e@.
data RiskEvent = RiskEvent
  { eventName   :: !String
  , probability :: !Double
  , maxLoss     :: !Double
  } deriving (Show, Eq)

-- | Aggregated results after running the full simulation.
data SimulationResult = SimulationResult
  { trials           :: !Int
  , occurrences      :: !Int
  , totalLoss        :: !Double
  , meanLossPerTrial :: !Double
  , maxObservedLoss  :: !Double
  , var95            :: !Double
  } deriving (Show, Eq)

-- | Run a Monte Carlo risk simulation.
--
-- @simulate events numTrials seed@ performs @numTrials@ independent
-- trials.  In each trial every event is checked against a random
-- draw; if the event fires, a uniform loss in @[0, maxLoss)@ is
-- sampled and accumulated.
--
-- The 95th percentile Value at Risk (VaR95) is computed by sorting
-- all trial losses and indexing at @floor(numTrials * 0.95)@.
--
-- Note how the RNG state (@StdGen@) is threaded through every
-- function that needs randomness.  There is no global mutable state;
-- every random draw returns a fresh generator, guaranteeing
-- reproducibility for a given seed.
simulate :: [RiskEvent] -> Int -> Int -> SimulationResult
simulate events numTrials seed =
  let
    initialGen = mkStdGen seed

    -- Run all trials, accumulating stats and the per-trial loss list.
    -- We use a strict left fold (go) to avoid building thunks.
    (!finalOcc, !finalTotal, !finalMax, !losses, !_finalGen) =
      go numTrials initialGen 0 0 0.0 []

    -- Sort trial losses for VaR calculation.
    -- Data.List.sort is an O(n log n) merge sort, stable and pure.
    sortedLosses = sort losses

    -- 95th percentile index, matching the Rust implementation:
    --   (numTrials as f64 * 0.95) as usize
    varIdx = floor (fromIntegral numTrials * (0.95 :: Double))
    varValue
      | varIdx < length sortedLosses = sortedLosses !! varIdx
      | otherwise                    = 0.0

    meanLoss
      | numTrials > 0 = finalTotal / fromIntegral numTrials
      | otherwise      = 0.0
  in
    SimulationResult
      { trials           = numTrials
      , occurrences      = finalOcc
      , totalLoss        = finalTotal
      , meanLossPerTrial = meanLoss
      , maxObservedLoss  = finalMax
      , var95            = varValue
      }
  where
    -- Manual tail-recursive loop.  Each iteration represents one trial.
    --
    -- We prepend trial losses to an accumulator list (O(1) cons) and
    -- sort once at the end, exactly like the Rust version pushes to a
    -- Vec and sorts after the loop.
    --
    -- BangPatterns on the accumulators ensure we evaluate eagerly,
    -- preventing a chain of unevaluated additions from consuming the
    -- entire heap (a classic Haskell space leak).
    go :: Int -> StdGen -> Int -> Double -> Double -> [Double]
       -> (Int, Double, Double, [Double], StdGen)
    go 0 gen !occ !total !maxObs acc = (occ, total, maxObs, acc, gen)
    go !remaining gen !occ !total !maxObs acc =
      let
        -- Process every event within this single trial.
        -- 'foldEvent' threads the RNG and accumulates the trial loss
        -- and occurrence count.
        (!trialLoss, !trialOcc, !gen') = foldEvents gen events 0.0 0

        -- Update running totals.
        !newTotal  = total + trialLoss
        !newOcc    = occ + trialOcc
        !newMax    = if trialLoss > maxObs then trialLoss else maxObs
      in
        go (remaining - 1) gen' newOcc newTotal newMax (trialLoss : acc)

    -- Fold over the event list for a single trial.
    --
    -- This is where Haskell's explicit RNG threading is most visible.
    -- In Rust you write @rng.gen::<f64>()@ and the borrow checker
    -- tracks the mutation.  Here we pass @gen@ in and get @gen'@ out,
    -- making the data flow completely explicit.
    foldEvents :: StdGen -> [RiskEvent] -> Double -> Int
               -> (Double, Int, StdGen)
    foldEvents gen [] !loss !occ = (loss, occ, gen)
    foldEvents gen (e : es) !loss !occ =
      -- Draw a probability check value in [0, 1).
      -- randomR is inclusive on both ends, but for Double the
      -- probability of hitting exactly 1.0 is negligible, matching
      -- Rust's gen::<f64>() which produces [0, 1).
      let (!roll, !gen1) = randomR (0.0 :: Double, 1.0) gen
      in
        if roll < probability e
          then
            -- Event fired. Sample a uniform loss in [0, maxLoss).
            let (!lossDraw, !gen2) = randomR (0.0 :: Double, maxLoss e) gen1
                !newLoss = loss + lossDraw
            in  foldEvents gen2 es newLoss (occ + 1)
          else
            -- Event did not fire. Pass the generator along unchanged
            -- (apart from the single draw we already consumed).
            foldEvents gen1 es loss occ

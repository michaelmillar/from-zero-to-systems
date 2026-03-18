module Main where

import Test.Hspec
import RiskSampler

main :: IO ()
main = hspec $ do

  describe "RiskSampler.simulate" $ do

    -- Mirrors Rust: zero_probability_event_never_occurs
    it "zero probability event never occurs" $ do
      let events = [ RiskEvent "never" 0.0 1000000.0 ]
          result = simulate events 10000 42
      occurrences result `shouldBe` 0
      totalLoss result `shouldBe` 0.0

    -- Mirrors Rust: certain_event_always_occurs
    it "certain event always occurs" $ do
      let events = [ RiskEvent "always" 1.0 100.0 ]
          result = simulate events 1000 42
      occurrences result `shouldBe` 1000
      totalLoss result `shouldSatisfy` (> 0.0)

    -- Mirrors Rust: var_95_is_not_greater_than_max_possible_loss
    it "VaR95 does not exceed max possible loss" $ do
      let events = [ RiskEvent "flood" 0.1 50000.0 ]
          result = simulate events 100000 7
      var95 result `shouldSatisfy` (<= 50000.0)

    -- Mirrors Rust: mean_loss_is_consistent_with_probability
    it "mean loss converges to prob * maxLoss / 2 within 5%" $ do
      let prob    = 0.2
          ml      = 1000.0
          events  = [ RiskEvent "outage" prob ml ]
          result  = simulate events 500000 99
          expected = prob * ml / 2.0
          tolerance = expected * 0.05
      abs (meanLossPerTrial result - expected) `shouldSatisfy` (< tolerance)

module Main where

import Data.Time.Clock (getCurrentTime, diffUTCTime)
import RiskSampler

main :: IO ()
main = do
  let events =
        [ RiskEvent "Cyber attack"         0.05 500000.0
        , RiskEvent "Server outage"        0.15  50000.0
        , RiskEvent "Supply chain delay"   0.20  25000.0
        , RiskEvent "Regulatory fine"      0.02 1000000.0
        ]
      numTrials  = 100000
      iterations = 100
      seed       = 42

  start <- getCurrentTime

  -- Run the simulation `iterations` times, forcing evaluation of var95
  -- each round so the compiler cannot optimise the work away.
  let go :: Int -> Double -> IO Double
      go 0 !acc = return acc
      go n !acc = do
        let !r = simulate events numTrials seed
        -- Force a field read to ensure full evaluation.
        let !v = var95 r
        go (n - 1) (acc + v)

  _ <- go iterations 0.0

  end <- getCurrentTime

  let elapsed    = diffUTCTime end start
      totalNs    = realToFrac elapsed * (1e9 :: Double) :: Double
      meanNs     = totalNs / fromIntegral iterations
      totalMs    = totalNs / 1e6

  -- Emit JSON on stdout, matching the project benchmark convention.
  putStrLn $ concat
    [ "{\"ok\": true"
    , ", \"mean_ns\": " , show (round meanNs :: Integer)
    , ", \"summary\": \"100k trials in " , show (round totalMs :: Integer) , "ms\""
    , ", \"iterations\": " , show iterations
    , "}"
    ]

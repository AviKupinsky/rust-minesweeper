# Minesweeper Board Difficulty: Math & Probability

## How Difficulty Estimation Works

When generating a Minesweeper board, we want to control how likely it is to get a board of a certain difficulty (Easy, Medium, Hard) using a "generate → estimate → repeat" approach. The **expected number of attempts** to get a board of a given difficulty depends on the probability that a random board matches that difficulty.

---

## Probability Model

Let:
- **p_easy** = Probability a random board is "Easy"
- **p_medium** = Probability a random board is "Medium"
- **p_hard** = Probability a random board is "Hard"

The **expected number of attempts** to get a board of a given difficulty is:

- **E_easy = 1 / p_easy**
- **E_medium = 1 / p_medium**
- **E_hard = 1 / p_hard**

---

## Typical Probabilities (Empirical)

- For **random boards** (no bias), on an 8x8 board with 10 mines:
    - p_easy ≈ 0.05 (5%)
    - p_medium ≈ 0.60 (60%)
    - p_hard ≈ 0.35 (35%)

- For **biased boards** (using mine placement patterns):
    - p_easy can be increased to 0.3–0.7 (30–70%)
    - p_hard can be increased to 0.5–0.8 (50–80%)

---

## Expected Attempts Table

| Difficulty      | Probability (p) | Expected Attempts (1/p) | With Bias? | Notes                                      |
|-----------------|-----------------|-------------------------|------------|---------------------------------------------|
| Easy            | 0.05            | 20                      | No         | Random, rare to get logic-solvable board    |
| Easy            | 0.3–0.7         | 1.4–3.3                 | Yes        | Bias: much more likely                      |
| Medium          | 0.6             | 1.7                     | Either     | Most random boards are medium               |
| Hard            | 0.35            | 2.9                     | No         | Random, some guessing required              |
| Hard            | 0.5–0.8         | 1.25–2                  | Yes        | Bias: hard boards are much more common      |

---

## Why Biasing Works

- **Easy boards:** By spreading mines out and ensuring large open areas, you increase the chance that the board is logic-solvable (no guessing).
- **Hard boards:** By clustering mines and increasing density, you increase the chance that the board requires guessing or advanced logic.

This **increases p_easy or p_hard** and **reduces the expected number of attempts**.

---

## Practical Impact of Biasing

- **Without bias:**  
  - For easy boards, you may need to generate **20 or more boards on average** before finding one that is truly logic-solvable.
  - For hard boards, you may need about **3 attempts** on average.
- **With bias:**  
  - For easy boards, you often need to generate only **2 boards on average** to get a logic-solvable one.
  - For hard boards, you may need just **1–2 attempts** on average.

Biasing mine placement makes the process much more efficient and predictable, especially for easy boards.

---

## Summary

- The expected number of attempts to get a board of a given difficulty is **1 divided by the probability** of generating such a board.
- **Biasing mine placement** increases the probability for your target difficulty, making board generation much faster and more predictable.

---

*References:*
(how-to-generate-minesweeper-boards-without-guessing)
- [Minesweeper Probability and Logic](https://www.minesweeper.info/wiki/Probability)
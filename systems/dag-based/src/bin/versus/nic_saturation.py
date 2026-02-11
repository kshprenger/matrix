import matplotlib.pyplot as plt
import pandas as pd

sb_2000 = pd.read_csv("sparse_bullshark_2000.csv", sep=" ", header=None)
avg_sb_2000 = sb_2000.groupby(0)[3].mean().reset_index()
b_2000 = pd.read_csv("bullshark_2000.csv", sep=" ", header=None)
avg_b_2000 = b_2000.groupby(0)[2].mean().reset_index()[2].mean()
plt.errorbar(
    avg_sb_2000[0],
    (avg_sb_2000[3] * 8000) / (1024 * 1024),
    fmt="o-",
    color="green",
    label="Sparse Bullshark 2Gb/sec",
    capsize=5,
)

plt.axhline(
    y=(avg_b_2000 * 8000) / (1024 * 1024),
    color="black",
    linestyle="--",
    linewidth=2,
    label="Bullshark 2Gb/sec",
)


plt.xticks(avg_sb_2000[0])

plt.xlabel("Sample size")
plt.ylabel("Network card saturation Mb/sec")

plt.legend(loc="upper left")
plt.show()

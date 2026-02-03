import matplotlib.pyplot as plt
import numpy as np
import pandas as pd

secs = 3600


sb_5 = pd.read_csv("sparse_bullshark_5.csv", sep=" ", header=None)
avg_sb_5 = sb_5.groupby(0)[1].mean().reset_index()
std_sb_5 = sb_5.groupby(0)[1].std().reset_index()

sb_10 = pd.read_csv("sparse_bullshark_10.csv", sep=" ", header=None)
avg_sb_10 = sb_10.groupby(0)[1].mean().reset_index()
std_sb_10 = sb_10.groupby(0)[1].std().reset_index()

sb_20 = pd.read_csv("sparse_bullshark_20.csv", sep=" ", header=None)
avg_sb_20 = sb_20.groupby(0)[1].mean().reset_index()
std_sb_20 = sb_20.groupby(0)[1].std().reset_index()

plt.errorbar(
    avg_sb_5[0],
    avg_sb_5[1] / secs,
    fmt="o-",
    color="green",
    label="Sparse Bullshark 5Mb/sec",
    capsize=5,
)
plt.errorbar(
    avg_sb_10[0],
    avg_sb_10[1] / secs,
    fmt="o-",
    color="blue",
    label="Sparse Bullshark 10Mb/sec",
    capsize=5,
)
plt.errorbar(
    avg_sb_20[0],
    avg_sb_20[1] / secs,
    fmt="o-",
    color="red",
    label="Sparse Bullshark 20Mb/sec",
    capsize=5,
)


b_5 = pd.read_csv("bullshark_5.csv", sep=" ", header=None)
mean_b_5 = b_5[0].mean()

b_10 = pd.read_csv("bullshark_10.csv", sep=" ", header=None)
mean_b_10 = b_10[0].mean()

b_20 = pd.read_csv("bullshark_20.csv", sep=" ", header=None)
mean_b_20 = b_20[0].mean()

plt.axhline(
    y=mean_b_5 / secs,
    color="black",
    linestyle="--",
    linewidth=2,
    label="Bullshark 5Mb/sec",
)

plt.axhline(
    y=mean_b_10 / secs,
    color="orange",
    linestyle="--",
    linewidth=2,
    label="Bullshark 10Mb/sec",
)

plt.axhline(
    y=mean_b_20 / secs,
    color="purple",
    linestyle="--",
    linewidth=2,
    label="Bullshark 20Mb/sec",
)


plt.xticks(avg_sb_5[0])
y_min = 0
y_max = 200
plt.yticks(np.arange(y_min, y_max + 10, 10))

plt.xlabel("Sample size")
plt.ylabel("Blocks per second")

plt.legend(loc="upper right")
plt.show()

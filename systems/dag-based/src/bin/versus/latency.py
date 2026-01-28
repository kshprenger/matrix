import matplotlib.pyplot as plt
import numpy as np
import pandas as pd

secs = 3600

plt.figure(figsize=(14, 6))

sb_5 = pd.read_csv("sparse_bullshark_5.csv", sep=" ", header=None)
avg_sb_5 = sb_5.groupby(0)[2].mean().reset_index()

sb_10 = pd.read_csv("sparse_bullshark_10.csv", sep=" ", header=None)
avg_sb_10 = sb_10.groupby(0)[2].mean().reset_index()

sb_20 = pd.read_csv("sparse_bullshark_20.csv", sep=" ", header=None)
avg_sb_20 = sb_20.groupby(0)[2].mean().reset_index()

plt.plot(
    avg_sb_5[0],
    avg_sb_5[2] / secs,
    "o-",
    color="green",
    label="Sparse Bullshark 5Mb/sec",
)
plt.plot(
    avg_sb_10[0],
    avg_sb_10[2] / secs,
    "o-",
    color="blue",
    label="Sparse Bullshark 10Mb/sec",
)
plt.plot(
    avg_sb_20[0],
    avg_sb_20[2] / secs,
    "o-",
    color="red",
    label="Sparse Bullshark 20Mb/sec",
)

b_5 = pd.read_csv("bullshark_5.csv", sep=" ", header=None)
mean_b_5 = b_5[1].mean() / secs

b_10 = pd.read_csv("bullshark_10.csv", sep=" ", header=None)
mean_b_10 = b_10[1].mean() / secs

b_20 = pd.read_csv("bullshark_20.csv", sep=" ", header=None)
mean_b_20 = b_20[1].mean() / secs

plt.axhline(
    y=mean_b_5, color="black", linestyle="--", linewidth=2, label="Bullshark 5Mb/sec"
)

plt.axhline(
    y=mean_b_10, color="orange", linestyle="--", linewidth=2, label="Bullshark 10Mb/sec"
)

plt.axhline(
    y=mean_b_20, color="purple", linestyle="--", linewidth=2, label="Bullshark 20Mb/sec"
)

plt.xticks(avg_sb_5[0])


plt.xlabel("Sample size")
plt.ylabel("Latency (sec)")

plt.legend(loc="upper left")
plt.show()

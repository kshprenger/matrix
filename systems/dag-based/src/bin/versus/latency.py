import matplotlib.pyplot as plt
import pandas as pd

secs = 60

plt.figure(figsize=(14, 6))


sb_2000 = pd.read_csv("sparse_bullshark_2000.csv", sep=" ", header=None)
avg_sb_2000 = sb_2000.groupby(0)[2].mean().reset_index()

plt.plot(
    avg_sb_2000[0],
    avg_sb_2000[2] / secs,
    "o-",
    color="blue",
    label="Sparse Bullshark 2Gb/sec",
)

b_2000 = pd.read_csv("bullshark_2000.csv", sep=" ", header=None)
mean_b_2000 = b_2000[1].mean() / secs

plt.axhline(
    y=mean_b_2000,
    color="purple",
    linestyle="--",
    linewidth=2,
    label="Bullshark 2Gb/sec",
)


plt.xticks(avg_sb_2000[0])


plt.xlabel("Sample size")
plt.ylabel("Latency (sec)")

plt.legend(loc="upper left")
plt.show()

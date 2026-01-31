import matplotlib.pyplot as plt
import pandas as pd

secs = 3600

plt.figure(figsize=(14, 6))

sb_1 = pd.read_csv("sparse_bullshark_threshold_1.csv", sep=" ", header=None)
avg_sb_1 = sb_1.groupby(0)[2].mean().reset_index()

sb_11 = pd.read_csv("sparse_bullshark_threshold_1.1.csv", sep=" ", header=None)
avg_sb_11 = sb_11.groupby(0)[2].mean().reset_index()

sb_12 = pd.read_csv("sparse_bullshark_threshold_1.2.csv", sep=" ", header=None)
avg_sb_12 = sb_12.groupby(0)[2].mean().reset_index()

sb_13 = pd.read_csv("sparse_bullshark_threshold_1.3.csv", sep=" ", header=None)
avg_sb_13 = sb_13.groupby(0)[2].mean().reset_index()

sb_14 = pd.read_csv("sparse_bullshark_threshold_1.4.csv", sep=" ", header=None)
avg_sb_14 = sb_14.groupby(0)[2].mean().reset_index()

sb_15 = pd.read_csv("sparse_bullshark_threshold_1.5.csv", sep=" ", header=None)
avg_sb_15 = sb_15.groupby(0)[2].mean().reset_index()

sb_16 = pd.read_csv("sparse_bullshark_threshold_1.6.csv", sep=" ", header=None)
avg_sb_16 = sb_16.groupby(0)[2].mean().reset_index()

sb_17 = pd.read_csv("sparse_bullshark_threshold_1.7.csv", sep=" ", header=None)
avg_sb_17 = sb_17.groupby(0)[2].mean().reset_index()

sb_18 = pd.read_csv("sparse_bullshark_threshold_1.8.csv", sep=" ", header=None)
avg_sb_18 = sb_18.groupby(0)[2].mean().reset_index()

sb_19 = pd.read_csv("sparse_bullshark_threshold_1.9.csv", sep=" ", header=None)
avg_sb_19 = sb_19.groupby(0)[2].mean().reset_index()

sb_2 = pd.read_csv("sparse_bullshark_threshold_2.csv", sep=" ", header=None)
avg_sb_2 = sb_2.groupby(0)[2].mean().reset_index()

plt.plot(
    avg_sb_1[0],
    avg_sb_1[2] / secs,
    "o-",
    color="green",
    label="Sparse Bullshark f+1",
)

plt.plot(
    avg_sb_11[0],
    avg_sb_11[2] / secs,
    "o-",
    color="orange",
    label="Sparse Bullshark 1.1f+1",
)

plt.plot(
    avg_sb_12[0],
    avg_sb_12[2] / secs,
    "o-",
    color="orange",
    label="Sparse Bullshark 1.2f+1",
)

plt.plot(
    avg_sb_13[0],
    avg_sb_13[2] / secs,
    "o-",
    color="orange",
    label="Sparse Bullshark 1.3f+1",
)

plt.plot(
    avg_sb_14[0],
    avg_sb_14[2] / secs,
    "o-",
    color="orange",
    label="Sparse Bullshark 1.4f+1",
)

plt.plot(
    avg_sb_15[0],
    avg_sb_15[2] / secs,
    "o-",
    color="orange",
    label="Sparse Bullshark 1.5f+1",
)

plt.plot(
    avg_sb_16[0],
    avg_sb_16[2] / secs,
    "o-",
    color="orange",
    label="Sparse Bullshark 1.6f+1",
)

plt.plot(
    avg_sb_17[0],
    avg_sb_17[2] / secs,
    "o-",
    color="orange",
    label="Sparse Bullshark 1.7f+1",
)

plt.plot(
    avg_sb_18[0],
    avg_sb_18[2] / secs,
    "o-",
    color="orange",
    label="Sparse Bullshark 1.8f+1",
)

plt.plot(
    avg_sb_19[0],
    avg_sb_19[2] / secs,
    "o-",
    color="red",
    label="Sparse Bullshark 1.9f+1",
)

plt.plot(
    avg_sb_2[0],
    avg_sb_2[2] / secs,
    "o-",
    color="black",
    label="Sparse Bullshark 2f+1",
)

plt.xticks(avg_sb_1[0])


plt.xlabel("Sample size")
plt.ylabel("Latency (sec)")

plt.legend(loc="upper left")
plt.show()

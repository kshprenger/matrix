import matplotlib.cm as cm
import matplotlib.pyplot as plt
import numpy as np
import pandas as pd

secs = 10800
k_validators = 2000
plt.figure(figsize=(14, 6))

thresholds = ["1", "1.1", "1.2", "1.3", "1.4", "1.5", "1.6", "1.7", "1.8", "1.9", "2"]
n_lines = len(thresholds)

colors = cm.Blues(np.linspace(0.4, 1.0, n_lines))


for i, thresh in enumerate(thresholds):
    filename = f"sparse_bullshark_threshold_{thresh}.csv"
    df = pd.read_csv(filename, sep=" ", header=None)
    avg_df = df.groupby(0)[2].mean().reset_index()

    label = f"Sparse Bullshark {thresh}f+1" if thresh != "1" else "Sparse Bullshark f+1"

    plt.plot(
        avg_df[0],
        avg_df[2] / secs,
        "o-",
        color=colors[i],
        label=label,
    )

plt.xlabel("Sample size")
plt.ylabel("Latency (sec)")
plt.legend(loc="upper left")
plt.title(f"Sparse bullshark: Direct commit variation with {k_validators} validators")
plt.show()

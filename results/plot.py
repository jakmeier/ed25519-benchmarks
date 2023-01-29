import json
import matplotlib.pyplot as plt

def load_criterion_data(name):
    json_file = json.load(open("./criterion_data/" + name + ".json"))
    data_points_num = len(json_file["times"])
    data = [json_file["times"][i] / json_file["iters"][i] / 1000 for i in range(data_points_num) ]
    return data

def load_custom_data(name):
    json_files = [json.load(open("./custom_bench_data/" + name + str(index) + ".json")) for index in range(10)]
    return [[nanos / 1000 for nanos in file] for file in json_files]


def plot(title, target, data, max):
    target.violinplot(data,  points=20, widths=0.7, showmeans=True, showextrema=False, quantiles=[[0.1,0.9] for _ in range(10)], showmedians=True)
    target.set_ylim(50, max)
    target.set_xticklabels([])
    target.set_ylabel("Time [μs]")
    target.set_title(title, fontsize=16)

def figure(fig):
    fig.patch.set_alpha(0.8)
    fig.suptitle("Sampling ed25519 verify", fontsize=10, fontweight="bold")

byz = [load_criterion_data("byz_warmup" + str(i)) for i in range(10)]
byz_nowarmup = [load_criterion_data("byz_nowarmup" + str(i)) for i in range(10)]
traditional = [load_criterion_data("traditional" + str(i)) for i in range(10)]
byz_no_flush = load_custom_data("no_flush")
byz_overwrite = load_custom_data("overwrite")
byz_wbinv = load_custom_data("wbinv")


fig, subplots = plt.subplots(nrows=1, ncols=2, figsize=(10, 6))
figure(fig)
plot("Random Samples", subplots[0], traditional, 65)
plot("Byzantine Samples", subplots[1], byz, 65)
fig.savefig("criterion_sampling.svg")


fig, subplots = plt.subplots(nrows=1, ncols=2, figsize=(10, 6))
figure(fig)
plot("Byzantine Samples (warm-up)", subplots[0], byz, 65)
plot("Byzantine Samples (no warm-up)", subplots[1], byz_nowarmup, 65)
fig.suptitle("Sampling ed25519 verify", fontsize=10, fontweight="bold")
fig.savefig("criterion_sampling_no_warmup.svg")


# fig, subplots = plt.subplots(nrows=1, ncols=3, figsize=(10, 6))
fig, subplots = plt.subplots(nrows=1, ncols=2, figsize=(10, 6))
figure(fig)
plot("No cache flush", subplots[0], byz_no_flush, 70)
plot("Manual cache flush", subplots[1], byz_overwrite, 70)
# plot("Flush using WBINV", subplots[2], byz_wbinv, 100)
fig.suptitle("Invalidating caches", fontsize=10, fontweight="bold")
fig.savefig("custom_bench.svg")



plt.show()

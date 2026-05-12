"""Set up FABRIC nodes for assignment."""

from fabrictestbed_extensions.fablib.fablib import FablibManager
from fabrictestbed_extensions.fablib.node import Interface, Node

fablib = FablibManager(fabric_rc="./fabric_rc")

name = "prj"
image = "default_ubuntu_22"
site = "INDI"

slice = fablib.new_slice(name=name)

NUM_NODES = 21

all_nodes: list[Node] = []
nics: list[Interface] = []


def init_node(name: str) -> None:
    node = slice.add_node(name=name, image=image, cores=2, ram=4, disk=9, site=site)
    try:
        nic = node.add_component(model="NIC_Basic", name="iface1").get_interfaces()[0]  # type: ignore
    except KeyError:
        print("    expected nics to be list!")
        exit(1)
    all_nodes.append(node)
    nics.append(nic)


init_node("main")
for i in range(1, NUM_NODES):
    init_node(f"worker{i}")

slice.add_l2network(name="net", interfaces=nics)
slice.submit()

# CONFIGURE NET

main_node = slice.get_node("main")
worker_nodes = [slice.get_node(f"worker{i}") for i in range(1, NUM_NODES)]
all_nodes = [main_node] + worker_nodes

for i, node in enumerate(all_nodes):
    iface = node.get_interfaces()[0].get_os_interface()  # type: ignore
    node.execute(
        f"sudo ip addr add 10.0.0.{i + 1}/24 dev {iface} && sudo ip link set {iface} up"
    )

# INSTALL & DOWNLOAD

REPO_DIR = "~/prj"

install_th = [
    node.execute_thread(
        "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    )
    for node in all_nodes
]
download_th = [
    node.execute_thread(
        f"git clone https://github.com/andrew-chang-dewitt/maze-robot.git {REPO_DIR}"
    )
    for node in all_nodes
]
for t in install_th + download_th:
    t.result()  # type: ignore

# BUILD BINS

worker_build_threads = [
    node.execute_thread(f"cd {REPO_DIR} && cargo build -r --example dist-bot")
    for node in worker_nodes
]

main_node.execute(f"cd {REPO_DIR} && cargo build -r --example dist-maze")

for t in worker_build_threads:
    # this is actually a Future<Thread>, type signature on fablib.Node.execute_thread() is wrong
    t.result()  # type: ignore

# TODO: start maze & robot nodes
maze_port = 4000
maze_addr = f"10.0.0.1:{maze_port}"
swarm_port = 4001

main_th = main_node.execute_thread(
    f"cargo run --example dist-maze -- examples/test-maze.txt {maze_port}"
)
robot_ths = [
    node.execute_thread(f"cargo run --example dist-bot -- {maze_addr} {swarm_port}")
    for node in worker_nodes
]

# wait for results
main_th.result()  # type: ignore
for t in robot_ths:
    # this is actually a Future<Thread>, type signature on fablib.Node.execute_thread() is wrong
    t.result()  # type: ignore

fablib.delete_slice(name)

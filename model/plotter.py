import matplotlib.pyplot as plt
import matplotlib.gridspec as gridspec
from matplotlib.animation import FuncAnimation
from matplotlib.widgets import CheckButtons
import numpy as np
from collections import deque
import socket
import re
import struct


# TODO collect from drone
def data_denerator():
    time = 0
    while True:
        frame = [
            np.linspace(time, time + 0.05, 5),
            np.random.random((3, 5)),
            np.random.random((3, 5)),
            np.random.random((4, 5)),
            np.random.random((2, 5)),
        ]
        time += 0.05
        yield frame


class DrosixSink:
    def __init__(self, address: str = "0.0.0.0", port: int = 9000):
        self.socket = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
        self.socket.settimeout(0.03)
        self.socket.bind((address, port))
        self.open = True

    def __iter__(self):
        head = b""
        frame = np.empty((0, 18))
        while self.open:
            try:
                data = (self.socket.recv(1024)).strip().split(b"\n")
            except socket.timeout:
                yield np.empty((0, 18))
                continue
            for line in data:
                match = re.match(rb"\[(\d+\.\d+)\s*\].*MEASURES (.*)$", head + line)
                if match:
                    time = float(match.group(1))
                    measures = match.group(2)
                    if len(measures) != 68:
                        head = head + line
                        continue
                    else:
                        head = b""
                    measures = list(struct.unpack("<fff fff f IIII fff fff", measures))
                    frame = np.vstack((frame, [time] + measures))

                else:
                    print(f"LOG: {line.decode()}")
            if len(frame) >= 3:
                yield frame.T
                frame = np.empty((0, 18))


class Graph:
    def __init__(
        self, ax: plt.Axes, title: str, labels: [str], ylim: tuple[float, float]
    ):
        ax.set_title(title)
        ax.set_ylim(*ylim)
        ax.get_xaxis().set_visible(False)
        self.ax = ax
        self.lines = {}
        for label in labels:
            (self.lines[label],) = self.ax.plot([], [], label=label)
        rax = self.ax.inset_axes([1.0, 0.8, 0.15, 0.2])
        colors = [line.get_color() for line in self.lines.values()]
        self.check = CheckButtons(
            ax=rax,
            labels=labels,
            actives=[True] * len(labels),
            check_props={"facecolor": colors},
        )
        self.check.on_clicked(self.on_checkbox)
        self.previous_check = []

    def set_visible(self, visible: bool):
        self.ax.set_visible(visible)
        if visible:
            for i, state in enumerate(self.previous_check):
                self.check.set_active(i, state)
        else:
            self.previous_check = self.check.get_status()
            self.check.clear()

    def set_data(self, time: [float], data: [[float]]):
        for i, line in enumerate(self.lines.values()):
            line.set_data(time, data[i])
        limit = max(1, max(time))
        self.ax.set_xlim(limit - 1, limit)

    def on_checkbox(self, label: str | None):
        if label:
            line = self.lines[label]
            line.set_visible(not line.get_visible())
        else:
            for line in self.lines.values():
                line.set_visible(False)
        line.figure.canvas.draw_idle()


# TODO add better UI: color / name / units
class Plotter:
    def __init__(self, sink, window_size=500):
        self.sink = sink
        self.fig = plt.figure()
        self.fig.subplots_adjust(hspace=0.3, wspace=0.3)
        self.gs_full = gridspec.GridSpec(2, 2)
        self.gs_single = gridspec.GridSpec(1, 1)
        self.zoomed = False
        self.time = deque(maxlen=window_size)
        self.pos = [
            deque(maxlen=window_size),
            deque(maxlen=window_size),
            deque(maxlen=window_size),
        ]
        self.vel = [
            deque(maxlen=window_size),
            deque(maxlen=window_size),
            deque(maxlen=window_size),
        ]
        self.mot = [
            deque(maxlen=window_size),
            deque(maxlen=window_size),
            deque(maxlen=window_size),
            deque(maxlen=window_size),
        ]
        self.pid = [
            deque(maxlen=window_size),
            deque(maxlen=window_size),
            deque(maxlen=window_size),
            deque(maxlen=window_size),
            deque(maxlen=window_size),
            deque(maxlen=window_size),
        ]

        self.ax_pos = self.fig.add_subplot(self.gs_full[0], label="Position")
        self.ax_vel = self.fig.add_subplot(
            self.gs_full[1], label="Velocity", sharex=self.ax_pos
        )
        self.ax_mot = self.fig.add_subplot(
            self.gs_full[2], label="Motors", sharex=self.ax_pos
        )
        self.ax_pid = self.fig.add_subplot(
            self.gs_full[3], label="PIDs", sharex=self.ax_pos
        )

        self.graph = {}
        self.graph["Position"] = Graph(
            ax=self.ax_pos,
            title="Position",
            labels=["Roll", "Pitch", "Yaw"],
            ylim=(-1.6, 1.6),
        )
        self.graph["Velocity"] = Graph(
            ax=self.ax_vel,
            title="Velocity",
            labels=["Roll", "Pitch", "Yaw"],
            ylim=(-10, 10),
        )
        self.graph["Motors"] = Graph(
            ax=self.ax_mot,
            title="Motors",
            labels=["M0", "M1", "M2", "M3"],
            ylim=(150000, 400000),
        )
        self.graph["PIDs"] = Graph(
            ax=self.ax_pid,
            title="PIDs",
            labels=[
                "PID roll",
                "PID pitch",
                "PID yaw",
                "PID vroll",
                "PID vpitch",
                "PID vyaw",
            ],
            ylim=(-50000, 50000),
        )

        self.fig.canvas.mpl_connect("button_press_event", self.on_click)
        self.fig.canvas.mpl_connect("key_press_event", self.on_key)

        self.anim = FuncAnimation(
            self.fig,
            self.animate,
            frames=self.sink,
            interval=25,
            cache_frame_data=False,
            repeat=False,
            # blit=True,
        )
        self.running = True

    def on_click(self, event):
        # prevent our handler when user use widget from toolbar such as zoom
        mode = plt.get_current_fig_manager().toolbar.mode
        ax_zoom = event.inaxes
        if not self.zoomed and mode == "" and ax_zoom is not None:
            for label, graph in self.graph.items():
                if label != ax_zoom._label:
                    graph.set_visible(False)
                else:
                    graph.set_visible(True)
                    graph.ax.set_position(
                        self.gs_single[0].get_position(event.canvas.figure)
                    )
            self.zoomed = True
            plt.draw()

    def on_key(self, event):
        if event.key == "escape" and self.zoomed:
            for i, (label, graph) in enumerate(self.graph.items()):
                graph.set_visible(True)
                graph.ax.set_position(self.gs_full[i].get_position(event.canvas.figure))
            self.zoomed = False
            plt.draw()
        elif event.key == "q":
            plt.close(event.canvas.figure)
        elif event.key == " ":
            if self.running:
                self.anim.pause()
                self.running = False
            else:
                self.anim.resume()
                self.running = True

    def animate(self, frame):
        if len(frame) > 0:
            self.time.extend(frame[0])
            [self.pos[i].extend(frame[1 + i]) for i in range(0, 3)]
            [self.vel[i].extend(frame[4 + i]) for i in range(0, 3)]
            [self.mot[i].extend(frame[8 + i]) for i in range(0, 4)]
            [self.pid[i].extend(frame[12 + i]) for i in range(0, 6)]

            self.graph["Position"].set_data(self.time, self.pos)
            self.graph["Velocity"].set_data(self.time, self.vel)
            self.graph["Motors"].set_data(self.time, self.mot)
            self.graph["PIDs"].set_data(self.time, self.pid)

            artits = []
            for graph in self.graph.values():
                if graph.ax.get_visible():
                    artits += graph.ax.get_lines()
            return artits


if __name__ == "__main__":
    sink = DrosixSink()
    plotter = Plotter(sink)
    plt.show()

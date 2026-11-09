import QtQuick
import Quickshell
import Quickshell.Wayland
import ".."

// The screen saver, as Windows 7 runs one: after some minutes without
// input, over everything, until a key or the mouse moves. "Mystify" here is
// drawn for this desktop: two four-sided shapes whose corners bounce off
// the edges, each leaving a trail of fading copies in slowly changing
// colours.
PanelWindow {
    id: saver

    readonly property bool active: Settings.screensaver !== "none"
        && (idle.isIdle || Settings.screensaverPreview) && !LockState.locked

    visible: saver.active
    color: "black"
    anchors.top: true
    anchors.bottom: true
    anchors.left: true
    anchors.right: true
    exclusionMode: ExclusionMode.Ignore
    WlrLayershell.layer: WlrLayer.Overlay
    WlrLayershell.namespace: "no_blur"
    WlrLayershell.keyboardFocus: WlrKeyboardFocus.None

    IdleMonitor {
        id: idle
        enabled: Settings.screensaver !== "none"
        timeout: Math.max(1, Settings.screensaverMinutes) * 60
        respectInhibitors: true
    }

    // The preview ends when the mouse moves, after a moment to let go of
    // the button that started it.
    /// When the saver last showed, for the preview's moment of grace.
    property real shownAt: 0

    MouseArea {
        anchors.fill: parent
        hoverEnabled: true
        onPositionChanged: {
            if (Settings.screensaverPreview && Date.now() - saver.shownAt > 800)
                Settings.screensaverPreview = false;
        }
    }

    onActiveChanged: {
        if (saver.active) {
            saver.shownAt = Date.now();
            // Placed again on the first step, once the canvas has its size.
            shapes.corners = [];
        }
    }

    Canvas {
        id: shapes

        anchors.fill: parent

        // Two shapes of four corners, each with a speed; the trail keeps
        // the last positions.
        property var corners: []
        property var trails: [[], []]
        property real hue: 0

        function reset() {
            var list = [];
            for (var s = 0; s < 2; s++) {
                var shape = [];
                for (var c = 0; c < 4; c++) {
                    shape.push({
                        x: Math.random() * width,
                        y: Math.random() * height,
                        dx: (Math.random() * 2 + 1.5) * (Math.random() < 0.5 ? -1 : 1),
                        dy: (Math.random() * 2 + 1.5) * (Math.random() < 0.5 ? -1 : 1)
                    });
                }
                list.push(shape);
            }
            shapes.corners = list;
            shapes.trails = [[], []];
        }

        function step() {
            if (width <= 0 || height <= 0)
                return;
            if (shapes.corners.length === 0)
                shapes.reset();
            for (var s = 0; s < shapes.corners.length; s++) {
                var shape = shapes.corners[s];
                for (var c = 0; c < shape.length; c++) {
                    var p = shape[c];
                    p.x += p.dx;
                    p.y += p.dy;
                    if (p.x < 0 || p.x > width) {
                        p.dx = -p.dx;
                        p.x = Math.max(0, Math.min(width, p.x));
                    }
                    if (p.y < 0 || p.y > height) {
                        p.dy = -p.dy;
                        p.y = Math.max(0, Math.min(height, p.y));
                    }
                }
                var trail = shapes.trails[s];
                trail.unshift(shape.map(function (p) { return { x: p.x, y: p.y }; }));
                if (trail.length > 7)
                    trail.pop();
            }
            shapes.hue = (shapes.hue + 0.002) % 1;
            requestPaint();
        }

        onPaint: {
            var context = getContext("2d");
            context.fillStyle = "black";
            context.fillRect(0, 0, width, height);
            context.lineWidth = 1.6;
            for (var s = 0; s < shapes.trails.length; s++) {
                var trail = shapes.trails[s];
                var hue = (shapes.hue + s * 0.45) % 1;
                for (var t = 0; t < trail.length; t++) {
                    var copy = trail[t];
                    context.strokeStyle = Qt.hsla(hue, 0.9, 0.6, 1 - t / trail.length);
                    context.beginPath();
                    context.moveTo(copy[0].x, copy[0].y);
                    for (var c = 1; c < copy.length; c++)
                        context.lineTo(copy[c].x, copy[c].y);
                    context.closePath();
                    context.stroke();
                }
            }
        }
    }

    Timer {
        interval: 33
        repeat: true
        running: saver.active
        onTriggered: shapes.step()
    }
}

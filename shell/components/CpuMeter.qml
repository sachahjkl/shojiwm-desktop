import QtQuick
import ".."
import "../theme.js" as Theme

// The CPU Meter gadget of the desktop: two dials on a plate of dark glass,
// the processor on the left and the memory on the right, each with its
// percentage. Drag it anywhere on the desktop; it stays there. The plate is
// drawn here, in the manner of the Windows 7 gadgets, not copied from them.
Item {
    id: meter

    width: 196
    height: 124

    Component.onCompleted: SystemStats.active = true
    Component.onDestruction: SystemStats.active = false

    // A drop shadow, then the glass.
    Rectangle {
        anchors.fill: plate
        anchors.topMargin: 3
        anchors.leftMargin: 2
        anchors.rightMargin: -2
        anchors.bottomMargin: -4
        radius: 14
        color: "#55000000"
    }

    Rectangle {
        id: plate

        anchors.fill: parent
        radius: 14
        border.width: 1
        border.color: "#cc0b1620"
        gradient: Gradient {
            GradientStop { position: 0.0; color: "#e6455868" }
            GradientStop { position: 0.5; color: "#e61b2733" }
            GradientStop { position: 1.0; color: "#e60c141c" }
        }

        // The light line inside the outline, and the gloss on the upper half.
        Rectangle {
            anchors.fill: parent
            anchors.margins: 1
            radius: 13
            color: "transparent"
            border.width: 1
            border.color: "#40ffffff"
        }

        Rectangle {
            x: 2
            y: 2
            width: parent.width - 4
            height: parent.height * 0.45
            radius: 12
            gradient: Gradient {
                GradientStop { position: 0.0; color: "#38ffffff" }
                GradientStop { position: 1.0; color: "#00ffffff" }
            }
        }
    }

    component Dial: Item {
        id: dial

        property real value: 0
        property string label: ""
        property string detail: ""

        // A gauge from 7 to 5 o'clock: the track, then the value in a green
        // to red gradient.
        Canvas {
            id: canvas

            anchors.fill: parent
            property real shown: dial.value
            Behavior on shown {
                NumberAnimation { duration: Settings.motion(400); easing.type: Easing.OutCubic }
            }
            onShownChanged: requestPaint()

            onPaint: {
                var context = getContext("2d");
                context.reset();
                var cx = width / 2;
                var cy = height / 2 + 2;
                var r = Math.min(width, height) / 2 - 6;
                var start = Math.PI * 0.75;
                var sweep = Math.PI * 1.5;
                context.lineCap = "round";
                context.lineWidth = 6;
                context.strokeStyle = "rgba(0, 0, 0, 0.55)";
                context.beginPath();
                context.arc(cx, cy, r, start, start + sweep);
                context.stroke();
                context.lineWidth = 4;
                context.strokeStyle = "rgba(255, 255, 255, 0.12)";
                context.beginPath();
                context.arc(cx, cy, r, start, start + sweep);
                context.stroke();
                if (shown > 0.005) {
                    var gradient = context.createLinearGradient(cx - r, 0, cx + r, 0);
                    gradient.addColorStop(0, "#5fd35a");
                    gradient.addColorStop(0.6, "#f2c335");
                    gradient.addColorStop(1, "#e5463a");
                    context.strokeStyle = gradient;
                    context.lineWidth = 4;
                    context.beginPath();
                    context.arc(cx, cy, r, start, start + sweep * shown);
                    context.stroke();
                }
            }
        }

        Column {
            anchors.centerIn: parent
            anchors.verticalCenterOffset: 4
            spacing: -1

            Text {
                anchors.horizontalCenter: parent.horizontalCenter
                text: Math.round(dial.value * 100) + "%"
                color: "#ffffff"
                style: Text.Raised
                styleColor: "#80000000"
                font.family: Theme.fontFamily
                font.pixelSize: dial.width > 90 ? 18 : 14
                font.bold: true
            }

            Text {
                anchors.horizontalCenter: parent.horizontalCenter
                text: dial.label
                color: "#b8cbdc"
                font.family: Theme.fontFamily
                font.pixelSize: 10
            }
        }

        Text {
            anchors.horizontalCenter: parent.horizontalCenter
            anchors.bottom: parent.bottom
            anchors.bottomMargin: -2
            text: dial.detail
            color: "#8fa6ba"
            font.family: Theme.fontFamily
            font.pixelSize: 9
        }
    }

    Dial {
        x: 8
        y: 8
        width: 104
        height: 104
        value: SystemStats.cpu
        label: "CPU"
    }

    Dial {
        x: 116
        y: 26
        width: 72
        height: 72
        value: SystemStats.memory
        label: "RAM"
        detail: SystemStats.memoryTotal > 0
            ? (SystemStats.memoryUsed / 1073741824).toFixed(1) + " / "
                + Math.round(SystemStats.memoryTotal / 1073741824) + " GB"
            : ""
    }
}

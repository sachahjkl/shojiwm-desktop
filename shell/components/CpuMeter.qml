import QtQuick
import QtQuick.Shapes
import ".."

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
        radius: Scheme.corner(14)
        color: "#55000000"
    }

    Rectangle {
        id: plate

        anchors.fill: parent
        radius: Scheme.corner(14)
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
            radius: Scheme.corner(13)
            color: "transparent"
            border.width: 1
            border.color: "#40ffffff"
        }

        Rectangle {
            x: 2
            y: 2
            width: parent.width - 4
            height: parent.height * 0.45
            radius: Scheme.corner(12)
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
        // to red gradient. Shapes, which the GPU draws: a reading moves the
        // value without painting anything on the processor.
        readonly property real centreX: width / 2
        readonly property real centreY: height / 2 + 2
        readonly property real radius: Math.min(width, height) / 2 - 6
        property real shown: dial.value
        Behavior on shown {
            NumberAnimation { duration: Settings.motion(400); easing.type: Easing.OutCubic }
        }

        Shape {
            anchors.fill: parent
            preferredRendererType: Shape.CurveRenderer

            ShapePath {
                strokeWidth: 6
                strokeColor: "#8c000000"
                fillColor: "transparent"
                capStyle: ShapePath.RoundCap
                PathAngleArc {
                    centerX: dial.centreX
                    centerY: dial.centreY
                    radiusX: dial.radius
                    radiusY: dial.radius
                    startAngle: 135
                    sweepAngle: 270
                }
            }

            ShapePath {
                strokeWidth: 4
                strokeColor: "#1fffffff"
                fillColor: "transparent"
                capStyle: ShapePath.RoundCap
                PathAngleArc {
                    centerX: dial.centreX
                    centerY: dial.centreY
                    radiusX: dial.radius
                    radiusY: dial.radius
                    startAngle: 135
                    sweepAngle: 270
                }
            }
        }

        // The value: a band 4 px wide with round ends, filled with the
        // gradient across the dial.
        Shape {
            id: band

            readonly property real outer: dial.radius + 2
            readonly property real inner: dial.radius - 2
            readonly property real sweep: 270 * dial.shown
            readonly property real end: (135 + sweep) * Math.PI / 180

            anchors.fill: parent
            visible: dial.shown > 0.005
            preferredRendererType: Shape.CurveRenderer

            ShapePath {
                strokeColor: "transparent"
                fillGradient: LinearGradient {
                    x1: dial.centreX - dial.radius
                    y1: 0
                    x2: dial.centreX + dial.radius
                    y2: 0
                    GradientStop { position: 0.0; color: "#5fd35a" }
                    GradientStop { position: 0.6; color: "#f2c335" }
                    GradientStop { position: 1.0; color: "#e5463a" }
                }

                PathAngleArc {
                    centerX: dial.centreX
                    centerY: dial.centreY
                    radiusX: band.outer
                    radiusY: band.outer
                    startAngle: 135
                    sweepAngle: band.sweep
                }
                PathArc {
                    x: dial.centreX + Math.cos(band.end) * band.inner
                    y: dial.centreY + Math.sin(band.end) * band.inner
                    radiusX: 2
                    radiusY: 2
                }
                PathAngleArc {
                    centerX: dial.centreX
                    centerY: dial.centreY
                    radiusX: band.inner
                    radiusY: band.inner
                    startAngle: 135 + band.sweep
                    sweepAngle: -band.sweep
                    moveToStart: false
                }
                PathArc {
                    x: dial.centreX + Math.cos(Math.PI * 0.75) * band.outer
                    y: dial.centreY + Math.sin(Math.PI * 0.75) * band.outer
                    radiusX: 2
                    radiusY: 2
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
                font.family: Scheme.theme.fontFamily
                font.pixelSize: dial.width > 90 ? 18 : 14
                font.bold: true
            }

            Text {
                anchors.horizontalCenter: parent.horizontalCenter
                text: dial.label
                color: "#b8cbdc"
                font.family: Scheme.theme.fontFamily
                font.pixelSize: 10
            }
        }

        Text {
            anchors.horizontalCenter: parent.horizontalCenter
            anchors.bottom: parent.bottom
            anchors.bottomMargin: -2
            text: dial.detail
            color: "#8fa6ba"
            font.family: Scheme.theme.fontFamily
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

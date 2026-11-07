import QtQuick
import QtQuick.Controls.Basic
import ".."

// A vertical scroll bar drawn as Windows 7 draws one: a pale grey trough,
// an arrow button at each end, and a thumb with a soft horizontal gradient,
// a grip of three lines in its middle, that turns blue under the pointer.
//
//   ListView { ScrollBar.vertical: AeroScrollBar {} }
ScrollBar {
    id: bar

    readonly property int thickness: 17
    readonly property int arrow: 17
    readonly property bool lit: bar.hovered || bar.pressed

    implicitWidth: thickness
    padding: 0
    topPadding: arrow
    bottomPadding: arrow
    minimumSize: 0.08
    policy: ScrollBar.AsNeeded
    visible: bar.size < 1.0

    // The trough.
    background: Rectangle {
        implicitWidth: bar.thickness
        gradient: Gradient {
            orientation: Gradient.Horizontal
            GradientStop { position: 0.0; color: Scheme.dark ? "#26303a" : "#e3e3e3" }
            GradientStop { position: 0.5; color: Scheme.dark ? "#2c3743" : "#f1f1f1" }
            GradientStop { position: 1.0; color: Scheme.dark ? "#26303a" : "#e3e3e3" }
        }
        border.width: 1
        border.color: Scheme.dark ? "#1b232b" : "#d7d7d7"

        // The arrow buttons at each end.
        Repeater {
            model: [false, true]

            delegate: Rectangle {
                id: button

                required property bool modelData
                readonly property bool down: modelData

                x: 0
                y: down ? parent.height - bar.arrow : 0
                width: bar.thickness
                height: bar.arrow
                radius: 2
                border.width: 1
                border.color: area.containsMouse
                    ? "#3c7fb1" : (bar.lit ? "#9da8b2" : "transparent")
                gradient: Gradient {
                    orientation: Gradient.Horizontal
                    GradientStop {
                        position: 0.0
                        color: area.pressed ? "#c2e4f6" : area.containsMouse ? "#e3f4fc"
                            : bar.lit ? "#f3f3f3" : "transparent"
                    }
                    GradientStop {
                        position: 1.0
                        color: area.pressed ? "#a9d9f2" : area.containsMouse ? "#c6e8fa"
                            : bar.lit ? "#dcdcdc" : "transparent"
                    }
                }

                Text {
                    anchors.centerIn: parent
                    text: button.down ? "▼" : "▲"
                    color: area.containsMouse ? "#1c5180" : (Scheme.dark ? "#a7b4c0" : "#606060")
                    font.pixelSize: 7
                }

                MouseArea {
                    id: area
                    anchors.fill: parent
                    hoverEnabled: true
                    onClicked: button.down ? bar.increase() : bar.decrease()
                    // Held, it keeps scrolling.
                    onPressAndHold: repeat.start()
                    onReleased: repeat.stop()
                }

                Timer {
                    id: repeat
                    interval: 60
                    repeat: true
                    onTriggered: button.down ? bar.increase() : bar.decrease()
                }
            }
        }
    }

    // The thumb.
    contentItem: Rectangle {
        implicitWidth: bar.thickness
        radius: 3
        border.width: 1
        border.color: bar.pressed ? "#3c7fb1" : bar.hovered ? "#3c7fb1"
            : (Scheme.dark ? "#56687a" : "#9b9b9b")
        gradient: Gradient {
            orientation: Gradient.Horizontal
            GradientStop {
                position: 0.0
                color: bar.pressed ? "#cde9f8" : bar.hovered ? "#e3f4fc"
                    : (Scheme.dark ? "#4a5967" : "#f3f3f3")
            }
            GradientStop {
                position: 0.45
                color: bar.pressed ? "#b4def4" : bar.hovered ? "#d0ecfa"
                    : (Scheme.dark ? "#43515e" : "#e8e8e8")
            }
            GradientStop {
                position: 0.55
                color: bar.pressed ? "#9fd4f0" : bar.hovered ? "#bde4f8"
                    : (Scheme.dark ? "#3c4955" : "#d9d9d9")
            }
            GradientStop {
                position: 1.0
                color: bar.pressed ? "#8ccaec" : bar.hovered ? "#a9dcf6"
                    : (Scheme.dark ? "#36424d" : "#cdcdcd")
            }
        }

        // The light line inside the edge.
        Rectangle {
            anchors.fill: parent
            anchors.margins: 1
            radius: Math.max(0, parent.radius - 1)
            color: "transparent"
            border.width: 1
            border.color: "#66ffffff"
        }

        // The grip, when the thumb is tall enough to hold it.
        Column {
            anchors.centerIn: parent
            spacing: 1
            visible: parent.height > 20

            Repeater {
                model: 4
                delegate: Rectangle {
                    width: 7
                    height: 1
                    color: bar.lit ? "#3c7fb1" : (Scheme.dark ? "#22303c" : "#8e8e8e")
                }
            }
        }
    }
}

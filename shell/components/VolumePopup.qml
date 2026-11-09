import QtQuick
import Quickshell
import ".."
import "../theme.js" as Theme

// The volume panel a click on the speaker opens, as Windows 7 shows it: the
// device at the top, a vertical slider, the mute button under it, and the
// link to the mixer at the bottom.
PopupWindow {
    id: popup

    property Item anchorItem: null
    property bool open: false

    signal dismissed

    /// `true` while the pointer is over the panel.
    readonly property bool pointerInside: hoverArea.hovered

    readonly property int trackHeight: 150

    color: "transparent"
    visible: popup.open && popup.anchorItem !== null

    anchor.item: popup.anchorItem
    anchor.rect: Qt.rect(0, 0,
                         popup.anchorItem ? popup.anchorItem.width : 1,
                         popup.anchorItem ? popup.anchorItem.height : 1)
    anchor.edges: Edges.Top
    anchor.gravity: Edges.Top
    anchor.margins.bottom: Theme.popupGap + 2
    anchor.adjustment: PopupAdjustment.SlideX | PopupAdjustment.FlipY

    // Wider while a player shows its controls at the top.
    implicitWidth: media.visible ? 220 : 104
    implicitHeight: body.implicitHeight + footer.height + 20

    Rectangle {
        anchors.fill: parent
        radius: Scheme.popupRadius
        border.width: 1
        border.color: Scheme.popupOutline
        gradient: Gradient {
            GradientStop { position: 0.0; color: Scheme.popupTop }
            GradientStop { position: 1.0; color: Scheme.popupBottom }
        }
    }

    // The light line inside the outline, as every Windows 7 menu has.
    Rectangle {
        anchors.fill: parent
        anchors.margins: 1
        radius: Math.max(0, Scheme.popupRadius - 1)
        color: "transparent"
        border.width: 1
        border.color: Scheme.popupInnerLine
    }

    HoverHandler {
        id: hoverArea
    }

    Column {
        id: body

        anchors.top: parent.top
        anchors.left: parent.left
        anchors.right: parent.right
        anchors.topMargin: 10
        spacing: 8

        MediaControls {
            id: media
            width: parent.width - 20
            anchors.horizontalCenter: parent.horizontalCenter
        }

        // The device: its icon and its name.
        Image {
            anchors.horizontalCenter: parent.horizontalCenter
            width: 32
            height: 32
            sourceSize: Qt.size(64, 64)
            source: Qt.resolvedUrl("../assets/" + (Scheme.dark ? "" : "ink-") + "speaker-3.svg")
        }

        Text {
            width: parent.width - 12
            anchors.horizontalCenter: parent.horizontalCenter
            horizontalAlignment: Text.AlignHCenter
            text: Audio.deviceName
            color: Scheme.text
            font.family: Theme.fontFamily
            font.pixelSize: Theme.fontSizeSmall
            elide: Text.ElideRight
        }

        // The slider: a sunken track, the level filled in green, and a
        // thumb. A press anywhere on it sets the volume there; the wheel
        // steps it.
        Item {
            id: slider

            anchors.horizontalCenter: parent.horizontalCenter
            width: 30
            height: popup.trackHeight

            readonly property real fraction: Audio.volume

            Rectangle {
                id: track
                anchors.horizontalCenter: parent.horizontalCenter
                anchors.top: parent.top
                anchors.bottom: parent.bottom
                anchors.topMargin: 5
                anchors.bottomMargin: 5
                width: 5
                radius: 2
                color: Scheme.dark ? "#1a232c" : "#e8edf2"
                border.width: 1
                border.color: Scheme.fieldBorder

                Rectangle {
                    anchors.left: parent.left
                    anchors.right: parent.right
                    anchors.bottom: parent.bottom
                    anchors.margins: 1
                    height: Math.max(0, (parent.height - 2) * slider.fraction)
                    radius: 1
                    color: Audio.muted ? Scheme.placeholder : "#2fb13f"
                }
            }

            // The thumb, a small raised bar across the track.
            Rectangle {
                anchors.horizontalCenter: parent.horizontalCenter
                y: track.y + (track.height - 2) * (1 - slider.fraction) + 1 - height / 2
                width: 22
                height: 10
                radius: 2
                border.width: 1
                border.color: sliderArea.pressed || sliderArea.containsMouse
                    ? Scheme.fieldFocus : Scheme.fieldBorder
                gradient: Gradient {
                    GradientStop { position: 0.0; color: sliderArea.containsMouse ? Scheme.buttonHotTop : Scheme.buttonTop }
                    GradientStop { position: 1.0; color: sliderArea.containsMouse ? Scheme.buttonHotBottom : Scheme.buttonBottom }
                }
            }

            MouseArea {
                id: sliderArea
                anchors.fill: parent
                hoverEnabled: true
                preventStealing: true

                function setFrom(y) {
                    var span = track.height - 2;
                    Audio.setVolume(1 - (y - track.y - 1) / span);
                }

                onPressed: function (event) { setFrom(event.y); }
                onPositionChanged: function (event) {
                    if (pressed)
                        setFrom(event.y);
                }
                onWheel: function (event) {
                    Audio.step(event.angleDelta.y / 120 * 2);
                }
            }
        }

        Text {
            anchors.horizontalCenter: parent.horizontalCenter
            text: Audio.percent
            color: Scheme.textMuted
            font.family: Theme.fontFamily
            font.pixelSize: Theme.fontSizeSmall
        }

        // The mute button: the speaker, with the red mark when muted.
        Rectangle {
            anchors.horizontalCenter: parent.horizontalCenter
            width: 30
            height: 26
            radius: Scheme.buttonRadius
            color: muteArea.containsMouse ? Scheme.hover : "transparent"
            border.width: muteArea.containsMouse ? 1 : 0
            border.color: Scheme.selectionBorder

            Image {
                anchors.centerIn: parent
                width: 18
                height: 18
                sourceSize: Qt.size(36, 36)
                source: Qt.resolvedUrl("../assets/" + (Scheme.dark ? "" : "ink-")
                    + "speaker-" + Audio.level + ".svg")
            }

            MouseArea {
                id: muteArea
                anchors.fill: parent
                hoverEnabled: true
                onClicked: Audio.toggleMute()
            }
        }
    }

    // The link to the mixer, on the band at the bottom.
    Rectangle {
        id: footer

        anchors.left: parent.left
        anchors.right: parent.right
        anchors.bottom: parent.bottom
        anchors.margins: 1
        height: 28
        radius: Math.max(0, Scheme.popupRadius - 1)
        color: Scheme.dark ? "#33000000" : "#80ffffff"

        Rectangle {
            anchors.top: parent.top
            anchors.left: parent.left
            anchors.right: parent.right
            height: 1
            color: Scheme.menuDivider
        }

        Text {
            anchors.centerIn: parent
            text: "Mixer"
            color: Scheme.link
            font.family: Theme.fontFamily
            font.pixelSize: Theme.fontSize
            font.underline: mixerArea.containsMouse

            MouseArea {
                id: mixerArea
                anchors.fill: parent
                anchors.margins: -4
                hoverEnabled: true
                cursorShape: Qt.PointingHandCursor
                onClicked: {
                    Audio.openMixer();
                    popup.dismissed();
                }
            }
        }
    }
}

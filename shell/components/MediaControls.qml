pragma ComponentBehavior: Bound

import QtQuick
import Quickshell
import Quickshell.Services.Mpris
import ".."
import "../theme.js" as Theme

// The player that plays now, or the first one: its cover, the title and
// the artist, and the previous, play or pause, and next buttons. Any
// program with an MPRIS interface counts (browsers, Spotify, mpv). Nothing
// shows while no program plays media.
Column {
    id: controls

    /// The player shown: the one that plays, else the first.
    readonly property MprisPlayer player: {
        var all = Mpris.players ? Mpris.players.values : [];
        for (var i = 0; i < all.length; i++) {
            if (all[i].isPlaying)
                return all[i];
        }
        return all.length > 0 ? all[0] : null;
    }

    visible: controls.player !== null
    spacing: 6

    Row {
        width: parent.width
        spacing: 8

        // The cover, or the player's icon.
        Rectangle {
            width: 40
            height: 40
            radius: Scheme.corner(2)
            color: Scheme.dark ? "#33ffffff" : "#1a2c4a66"
            border.width: 1
            border.color: Scheme.fieldBorder
            clip: true

            Image {
                id: cover
                anchors.fill: parent
                anchors.margins: 1
                fillMode: Image.PreserveAspectCrop
                sourceSize: Qt.size(80, 80)
                source: controls.player ? controls.player.trackArtUrl : ""
                visible: status === Image.Ready
            }

            ShellIcon {
                anchors.centerIn: parent
                implicitSize: 24
                visible: !cover.visible
                source: controls.player
                    ? Quickshell.iconPath(controls.player.desktopEntry || "", "audio-x-generic")
                    : ""
            }
        }

        Column {
            width: parent.width - 48
            anchors.verticalCenter: parent.verticalCenter
            spacing: 1

            Text {
                width: parent.width
                text: controls.player
                    ? controls.player.trackTitle || controls.player.identity : ""
                color: Scheme.text
                font.family: Theme.fontFamily
                font.pixelSize: Theme.fontSize
                font.bold: true
                elide: Text.ElideRight
            }

            Text {
                width: parent.width
                text: controls.player
                    ? controls.player.trackArtist || controls.player.identity : ""
                color: Scheme.textMuted
                font.family: Theme.fontFamily
                font.pixelSize: Theme.fontSizeSmall
                elide: Text.ElideRight
            }
        }
    }

    Row {
        anchors.horizontalCenter: parent.horizontalCenter
        spacing: 6

        component MediaButton: Rectangle {
            id: mediaButton

            property string glyph: ""
            property bool available: true
            signal activated

            width: 34
            height: 26
            radius: Scheme.buttonRadius
            opacity: mediaButton.available ? 1 : 0.4
            border.width: 1
            border.color: buttonArea.containsMouse ? Scheme.selectionBorder : Scheme.fieldBorder
            gradient: Gradient {
                GradientStop {
                    position: 0.0
                    color: buttonArea.pressed ? Scheme.buttonPressedTop
                        : buttonArea.containsMouse ? Scheme.buttonHotTop : Scheme.buttonTop
                }
                GradientStop {
                    position: 1.0
                    color: buttonArea.pressed ? Scheme.buttonPressedBottom
                        : buttonArea.containsMouse ? Scheme.buttonHotBottom : Scheme.buttonBottom
                }
            }

            Image {
                anchors.centerIn: parent
                width: 14
                height: 14
                sourceSize: Qt.size(28, 28)
                source: Qt.resolvedUrl("../assets/" + (Scheme.dark ? "" : "ink-")
                    + "media-" + mediaButton.glyph + ".svg")
            }

            MouseArea {
                id: buttonArea
                anchors.fill: parent
                hoverEnabled: true
                enabled: mediaButton.available
                onClicked: mediaButton.activated()
            }
        }

        MediaButton {
            glyph: "previous"
            available: controls.player !== null && controls.player.canGoPrevious
            onActivated: controls.player.previous()
        }

        MediaButton {
            glyph: controls.player && controls.player.isPlaying ? "pause" : "play"
            available: controls.player !== null && controls.player.canTogglePlaying
            onActivated: controls.player.togglePlaying()
        }

        MediaButton {
            glyph: "next"
            available: controls.player !== null && controls.player.canGoNext
            onActivated: controls.player.next()
        }
    }

    Rectangle {
        width: parent.width
        height: 1
        color: Scheme.menuDivider
    }
}

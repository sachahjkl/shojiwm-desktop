pragma ComponentBehavior: Bound

import QtQuick
import Quickshell
import Quickshell.Services.Mpris
import Quickshell.Wayland
import ".."
import "../theme.js" as Theme

// The live previews Windows 7 shows over a taskbar button.
//
// `ScreencopyView` captures one window. Quickshell captures a window through
// `hyprland_toplevel_export_manager_v1`, which ShojiWM implements for exactly
// that reason (src/shojiwm_lib/src/protocols/hyprland_toplevel_export.rs).
PopupWindow {
    id: popup

    property var windows: []
    property var anchorWindow: null
    property rect anchorRect: Qt.rect(0, 0, 1, 1)

    signal dismissed

    readonly property int count: windows.length

    /// The media player of these windows' program, for the thumbnail
    /// toolbar Windows 7 gives media players: the one whose .desktop file
    /// is the program's.
    readonly property MprisPlayer player: {
        if (popup.count === 0 || !Mpris.players)
            return null;
        var appId = String(popup.windows[0].appId || "").toLowerCase();
        var players = Mpris.players.values;
        for (var i = 0; i < players.length; i++) {
            var entry = String(players[i].desktopEntry || "").toLowerCase();
            if (entry.length > 0 && (appId === entry || appId.indexOf(entry) >= 0
                                     || entry.indexOf(appId) >= 0))
                return players[i];
        }
        return null;
    }
    readonly property int toolbarHeight: popup.player ? 34 : 0

    color: "transparent"
    visible: count > 0

    anchor.window: popup.anchorWindow
    anchor.rect: popup.anchorRect
    anchor.edges: Edges.Top
    anchor.gravity: Edges.Top
    anchor.margins.bottom: Theme.popupGap
    anchor.adjustment: PopupAdjustment.SlideX | PopupAdjustment.FlipY

    implicitWidth: Theme.thumbPadding * 2
        + count * Theme.thumbWidth
        + Math.max(0, count - 1) * Theme.thumbGap
    implicitHeight: Theme.thumbPadding * 2
        + Theme.thumbHeight + Theme.thumbTitleHeight + popup.toolbarHeight

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
        radius: Scheme.corner(Math.max(0, Scheme.popupRadius - 1))
        color: "transparent"
        border.width: 1
        border.color: Scheme.popupInnerLine
    }

    /// `true` while the pointer is over the popup, so the taskbar can keep
    /// it up when the pointer leaves the button through it.
    readonly property bool pointerInside: hoverArea.hovered

    // A hover handler, not a MouseArea: it sees the pointer over the rows
    // and icons too, which take the hover from a MouseArea under them.
    HoverHandler {
        id: hoverArea
    }

    // The thumbnail toolbar: previous, play or pause, and next.
    Row {
        anchors.horizontalCenter: parent.horizontalCenter
        anchors.bottom: parent.bottom
        anchors.bottomMargin: Theme.thumbPadding
        spacing: 6
        visible: popup.player !== null

        component ToolButton: Rectangle {
            id: tool

            property string glyph: ""
            property bool available: true
            signal activated

            width: 34
            height: 26
            radius: Scheme.buttonRadius
            opacity: tool.available ? 1 : 0.4
            border.width: 1
            border.color: toolArea.containsMouse ? Scheme.selectionBorder : Scheme.fieldBorder
            gradient: Gradient {
                GradientStop {
                    position: 0.0
                    color: toolArea.pressed ? Scheme.buttonPressedTop
                        : toolArea.containsMouse ? Scheme.buttonHotTop : Scheme.buttonTop
                }
                GradientStop {
                    position: 1.0
                    color: toolArea.pressed ? Scheme.buttonPressedBottom
                        : toolArea.containsMouse ? Scheme.buttonHotBottom : Scheme.buttonBottom
                }
            }

            Image {
                anchors.centerIn: parent
                width: 14
                height: 14
                sourceSize: Qt.size(28, 28)
                source: Qt.resolvedUrl("../assets/" + (Scheme.dark ? "" : "ink-")
                    + "media-" + tool.glyph + ".svg")
            }

            MouseArea {
                id: toolArea
                anchors.fill: parent
                hoverEnabled: true
                enabled: tool.available
                onClicked: tool.activated()
            }
        }

        ToolButton {
            glyph: "previous"
            available: popup.player !== null && popup.player.canGoPrevious
            onActivated: popup.player.previous()
        }

        ToolButton {
            glyph: popup.player && popup.player.isPlaying ? "pause" : "play"
            available: popup.player !== null && popup.player.canTogglePlaying
            onActivated: popup.player.togglePlaying()
        }

        ToolButton {
            glyph: "next"
            available: popup.player !== null && popup.player.canGoNext
            onActivated: popup.player.next()
        }
    }

    Row {
        anchors.horizontalCenter: parent.horizontalCenter
        anchors.top: parent.top
        anchors.topMargin: Theme.thumbPadding
        spacing: Theme.thumbGap

        Repeater {
            model: popup.windows

            delegate: Item {
                id: card

                required property var modelData

                width: Theme.thumbWidth
                height: Theme.thumbHeight + Theme.thumbTitleHeight

                // Under the pointer, the title makes room for the close
                // button Windows 7 puts at the card's top right.
                readonly property bool hovered: cardHover.hovered

                HoverHandler {
                    id: cardHover
                }

                Text {
                    id: label

                    width: parent.width - (card.hovered ? 22 : 0)
                    height: Theme.thumbTitleHeight
                    text: Apps.titleOf(card.modelData)
                    color: Scheme.text
                    font.family: Theme.fontFamily
                    font.pixelSize: Theme.fontSizeSmall
                    elide: Text.ElideRight
                    verticalAlignment: Text.AlignVCenter
                }

                // The close button: red under the pointer, as the caption's.
                Rectangle {
                    anchors.right: parent.right
                    anchors.top: parent.top
                    anchors.topMargin: 1
                    width: 20
                    height: 17
                    radius: Scheme.corner(3)
                    visible: card.hovered
                    border.width: 1
                    border.color: closeArea.containsMouse ? "#7a1a12" : "transparent"
                    gradient: Gradient {
                        GradientStop { position: 0.0; color: closeArea.containsMouse ? "#f08a7c" : "transparent" }
                        GradientStop { position: 1.0; color: closeArea.containsMouse ? "#c42b1c" : "transparent" }
                    }

                    Text {
                        anchors.centerIn: parent
                        text: "×"
                        color: closeArea.containsMouse ? "#ffffff" : Scheme.textMuted
                        font.pixelSize: 15
                        font.bold: true
                    }

                    MouseArea {
                        id: closeArea
                        anchors.fill: parent
                        hoverEnabled: true
                        onClicked: card.modelData.close()
                    }
                }

                Rectangle {
                    anchors.top: label.bottom
                    width: parent.width
                    height: Theme.thumbHeight
                    color: Theme.thumbFrame
                    border.width: 1
                    border.color: cardArea.containsMouse
                        ? Scheme.selectionBorder : Scheme.popupOutline

                    // The live picture, when the compositor can hand one over.
                    Loader {
                        id: live
                        anchors.fill: parent
                        anchors.margins: 1
                        active: Theme.liveThumbnails

                        sourceComponent: ScreencopyView {
                            captureSource: card.modelData
                            live: true
                            paintCursor: false
                        }
                    }

                    // Otherwise, and until its first picture, the program's icon
                    // in the middle of the card: a capture that never delivers
                    // one (a compositor drawing in software) leaves no blank card.
                    ShellIcon {
                        anchors.centerIn: parent
                        implicitSize: 32
                        visible: !Theme.liveThumbnails || !(live.item as ScreencopyView) || !(live.item as ScreencopyView).hasContent
                        source: card.modelData
                            ? Apps.iconFor(card.modelData.appId) : ""
                    }

                    MouseArea {
                        id: cardArea

                        anchors.fill: parent
                        hoverEnabled: true
                        acceptedButtons: Qt.LeftButton | Qt.MiddleButton

                        onClicked: function (event) {
                            if (event.button === Qt.MiddleButton) {
                                card.modelData.close();
                            } else {
                                card.modelData.minimized = false;
                                card.modelData.activate();
                            }
                            popup.dismissed();
                        }
                    }
                }
            }
        }
    }
}

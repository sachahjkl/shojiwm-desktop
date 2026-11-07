pragma ComponentBehavior: Bound

import QtQuick
import Quickshell
import Quickshell.Wayland
import Quickshell.Widgets
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
        + Theme.thumbHeight + Theme.thumbTitleHeight

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

    /// `true` while the pointer is over the popup, so the taskbar can keep
    /// it up when the pointer leaves the button through it.
    readonly property bool pointerInside: hoverArea.hovered

    // A hover handler, not a MouseArea: it sees the pointer over the rows
    // and icons too, which take the hover from a MouseArea under them.
    HoverHandler {
        id: hoverArea
    }

    Row {
        anchors.centerIn: parent
        spacing: Theme.thumbGap

        Repeater {
            model: popup.windows

            delegate: Item {
                id: card

                required property var modelData

                width: Theme.thumbWidth
                height: Theme.thumbHeight + Theme.thumbTitleHeight

                Text {
                    id: label

                    width: parent.width
                    height: Theme.thumbTitleHeight
                    text: Apps.titleOf(card.modelData)
                    color: Scheme.text
                    font.family: Theme.fontFamily
                    font.pixelSize: Theme.fontSizeSmall
                    elide: Text.ElideRight
                    verticalAlignment: Text.AlignVCenter
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
                        anchors.fill: parent
                        anchors.margins: 1
                        active: Theme.liveThumbnails

                        sourceComponent: ScreencopyView {
                            captureSource: card.modelData
                            live: true
                            paintCursor: false
                        }
                    }

                    // Otherwise the program's icon, in the middle of the card.
                    IconImage {
                        anchors.centerIn: parent
                        implicitSize: 32
                        visible: !Theme.liveThumbnails
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

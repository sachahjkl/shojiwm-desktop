pragma ComponentBehavior: Bound

import QtQuick
import Quickshell
import ".."

// A right-click menu of the taskbar, above what was clicked: the bell, the
// volume, the network, the clock, the show-desktop strip, the bar itself.
//
// `entries` lists its lines, each an object:
//   label     the text
//   icon      a picture in front (optional)
//   separator a line between groups, with nothing else (optional)
//   enabled   false greys it out (optional, true by default)
//   checked   true or false gives it a check box (optional)
//   run       what it does; the menu closes first
PopupWindow {
    id: popup

    property bool open: false
    property var anchorWindow: null
    property rect anchorRect: Qt.rect(0, 0, 1, 1)
    property var entries: []

    signal dismissed

    color: "transparent"
    visible: popup.open

    anchor.window: popup.anchorWindow
    anchor.rect: popup.anchorRect
    anchor.edges: Scheme.awayFromBar
    anchor.gravity: Scheme.awayFromBar
    anchor.margins.bottom: Scheme.popupMargin(Scheme.theme.popupGap)
    anchor.adjustment: PopupAdjustment.SlideX | PopupAdjustment.FlipY

    /// A line has an icon or a check box: every line keeps that column, as
    /// Windows 7 menus do, so the labels line up.
    readonly property bool gutter: popup.entries.some(function (entry) {
        return !!entry.icon || entry.checked !== undefined;
    })

    readonly property real longestLabel: {
        var longest = 0;
        for (var i = 0; i < popup.entries.length; i++) {
            if (popup.entries[i].label)
                longest = Math.max(longest, metrics.advanceWidth(popup.entries[i].label));
        }
        return Math.ceil(longest);
    }

    implicitWidth: Math.min(Scheme.theme.popupMaxWidth,
        Math.max(Scheme.theme.popupMinWidth, popup.longestLabel + Scheme.theme.popupPadX * 2 + 24
            + (popup.gutter ? 16 + 7 : 0)))
    implicitHeight: column.implicitHeight + Scheme.theme.popupPadY * 2 + 2

    FontMetrics {
        id: metrics
        font.family: Scheme.theme.fontFamily
        font.pixelSize: Scheme.theme.fontSize
    }

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

    Column {
        id: column

        anchors.left: parent.left
        anchors.right: parent.right
        anchors.top: parent.top
        anchors.topMargin: Scheme.theme.popupPadY
        anchors.leftMargin: 1
        anchors.rightMargin: 1

        Repeater {
            model: popup.entries

            delegate: MenuItemRow {
                required property var modelData

                width: column.width
                label: modelData.label || ""
                icon: modelData.icon || ""
                separator: !!modelData.separator
                itemEnabled: modelData.enabled !== false
                checkable: modelData.checked !== undefined
                checked: modelData.checked === true
                gutter: popup.gutter
                onActivated: {
                    popup.dismissed();
                    if (modelData.run)
                        modelData.run();
                }
            }
        }
    }
}

pragma ComponentBehavior: Bound

import QtQuick
import Quickshell
import ".."

// The menu of a notification-area item, drawn like a Windows 7 menu.
//
// `QsMenuOpener` opens the item's DBusMenu and lists its entries
// (Quickshell/QsMenuOpener, Quickshell/QsMenuEntry). An entry with children
// is itself a menu: resting on it, or clicking it, opens this same component
// beside the line, through a LazyLoader, as deep as the item goes.
PopupWindow {
    id: popup

    /// The tray item whose menu this is. A submenu leaves it null and sets
    /// `menu` instead.
    property var item: null
    property var menu: item ? item.menu : null
    property var anchorWindow: null
    property rect anchorRect: Qt.rect(0, 0, 1, 1)
    /// `true` for a menu opened from a line of another one.
    property bool submenu: false

    signal dismissed

    readonly property var entries: opener.children ? opener.children.values : []

    /// The entry whose submenu is open, or null.
    property var openEntry: null
    property rect openRect: Qt.rect(0, 0, 1, 1)

    onMenuChanged: popup.openEntry = null
    onVisibleChanged: {
        if (!visible)
            popup.openEntry = null;
    }

    color: "transparent"
    visible: menu !== null && menu !== undefined && entries.length > 0

    anchor.window: popup.anchorWindow
    anchor.rect: popup.anchorRect
    // The root menu stands on the tray icon; a submenu hangs from the right
    // of its line, and moves to the left when the screen ends.
    anchor.edges: popup.submenu ? Edges.Right | Edges.Top : Edges.Top
    anchor.gravity: popup.submenu ? Edges.Right | Edges.Bottom : Edges.Top
    anchor.margins.bottom: popup.submenu ? 0 : Scheme.theme.popupGap
    anchor.margins.top: popup.submenu ? -Scheme.theme.popupPadY - 1 : 0
    anchor.adjustment: popup.submenu
        ? PopupAdjustment.FlipX | PopupAdjustment.SlideY
        : PopupAdjustment.SlideX | PopupAdjustment.FlipY

    implicitWidth: Math.min(
        Scheme.theme.popupMaxWidth,
        Math.max(Scheme.theme.popupMinWidth, popup.widest + Scheme.theme.popupPadX * 2 + 24))
    implicitHeight: column.implicitHeight + Scheme.theme.popupPadY * 2 + 2

    QsMenuOpener {
        id: opener
        menu: popup.menu
    }

    FontMetrics {
        id: metrics
        font.family: Scheme.theme.fontFamily
        font.pixelSize: Scheme.theme.fontSize
    }

    // The popup is sized before it is shown, so the width comes from the
    // entry strings rather than from laid-out rows.
    readonly property real widest: {
        var width = 0;
        for (var i = 0; i < entries.length; i++) {
            var text = entries[i].text || "";
            width = Math.max(width, metrics.advanceWidth(text));
        }
        return width;
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

    /// The open submenu, a TrayMenu, or null.
    readonly property var childMenu: child.item

    // The line the pointer rests on, and the timer that opens its submenu.
    property var restingEntry: null
    property rect restingRect: Qt.rect(0, 0, 1, 1)

    Timer {
        id: submenuTimer
        interval: Scheme.theme.submenuDelay
        onTriggered: {
            popup.openEntry = popup.restingEntry;
            popup.openRect = popup.restingRect;
        }
    }

    function rest(entry, rect) {
        popup.restingEntry = entry;
        popup.restingRect = rect;
        if (popup.openEntry !== entry)
            submenuTimer.restart();
    }

    function openSubmenu(entry, rect) {
        submenuTimer.stop();
        popup.openEntry = entry;
        popup.openRect = rect;
    }

    // Quickshell's loader, not Qt's: what it loads is a window, not an item.
    LazyLoader {
        id: child

        active: popup.openEntry !== null
        source: "TrayMenu.qml"

        onItemChanged: {
            var menu = popup.childMenu;
            if (!menu)
                return;
            menu.submenu = true;
            menu.anchorWindow = popup;
            menu.anchorRect = Qt.binding(function () {
                return popup.openRect;
            });
            menu.menu = Qt.binding(function () {
                return popup.openEntry;
            });
            menu.dismissed.connect(popup.dismissed);
        }
    }

    // A hover handler, not a MouseArea: it sees the pointer over the rows
    // and icons too, which take the hover from a MouseArea under them.

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

                id: entryRow

                width: column.width
                separator: modelData ? modelData.isSeparator : false
                itemEnabled: modelData ? modelData.enabled : false
                label: modelData && modelData.text ? modelData.text : ""
                icon: modelData && modelData.icon ? modelData.icon : ""
                hasChildren: modelData ? modelData.hasChildren : false
                highlighted: popup.openEntry !== null
                    && popup.openEntry === modelData
                checkable: modelData
                    ? modelData.buttonType !== QsMenuButtonType.None : false
                checked: modelData ? modelData.checkState === Qt.Checked : false

                onHoveredChanged: {
                    if (!entryRow.hovered)
                        return;
                    if (entryRow.hasChildren && entryRow.itemEnabled) {
                        popup.rest(modelData, entryRow.rowRect());
                    } else {
                        // Resting on a plain line closes an open submenu.
                        popup.restingEntry = null;
                        submenuTimer.restart();
                    }
                }

                onActivated: {
                    if (!modelData)
                        return;
                    if (entryRow.hasChildren) {
                        popup.openSubmenu(modelData, entryRow.rowRect());
                        return;
                    }
                    modelData.triggered();
                    popup.dismissed();
                }
            }
        }
    }
}

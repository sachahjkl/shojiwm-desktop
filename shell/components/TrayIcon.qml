import QtQuick
import Quickshell.Widgets
import ".."
import "../theme.js" as Theme

// One notification-area icon: on the taskbar, or in the popover of hidden
// icons. A left click activates the item, a right click asks for its menu.
// An item that offers nothing but a menu opens it on either button.
Item {
    id: icon

    property var item: null
    /// The tint of the hover band: white on the taskbar's dark glass, the
    /// selection blue in the pale popover.
    property color hoverColor: "#2effffff"

    /// `rect` is where the icon is in its window.
    signal menuRequested(var item, rect at)
    signal hoverChanged(bool inside)
    /// The item was activated, so a popover holding it can close.
    signal used

    width: Theme.trayIconSize + 8
    height: Theme.trayIconSize + 8

    function iconRect() {
        var point = icon.mapToItem(null, 0, 0);
        return Qt.rect(point.x, point.y, icon.width, icon.height);
    }

    Rectangle {
        anchors.fill: parent
        radius: Scheme.itemRadius
        color: icon.hoverColor
        opacity: area.containsMouse ? 1 : 0
        Behavior on opacity {
            NumberAnimation { duration: Theme.fade }
        }
    }

    IconImage {
        anchors.centerIn: parent
        implicitSize: Theme.trayIconSize
        source: icon.item && icon.item.icon ? icon.item.icon : ""
    }

    MouseArea {
        id: area
        anchors.fill: parent
        hoverEnabled: true
        acceptedButtons: Qt.LeftButton | Qt.RightButton

        onEntered: icon.hoverChanged(true)
        onExited: icon.hoverChanged(false)

        onClicked: function (event) {
            var item = icon.item;
            if (!item)
                return;
            var wantsMenu = event.button === Qt.RightButton || item.onlyMenu;
            if (wantsMenu) {
                if (item.hasMenu)
                    icon.menuRequested(item, icon.iconRect());
                else if (!item.onlyMenu) {
                    item.secondaryActivate();
                    icon.used();
                }
                return;
            }
            item.activate();
            icon.used();
        }
    }
}

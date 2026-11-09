import QtQuick
import Quickshell.Widgets
import ".."
import "../theme.js" as Theme

// One notification-area icon: on the taskbar, or in the popover of hidden
// icons. A left click activates the item, a right click asks for its menu.
// An item that offers nothing but a menu opens it on either button. Dragged,
// it carries its item id to the taskbar or the popover, which show or hide
// it.
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
    /// The icon is being dragged; a popover holding it stays open meanwhile.
    readonly property bool dragging: area.drag.active

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

    // What the pointer drags: invisible, the drag shows a picture of the
    // icon. A platform drag, so it reaches the other surfaces of the shell.
    Item {
        id: dragProxy

        width: icon.width
        height: icon.height
        Drag.active: area.drag.active
        Drag.dragType: Drag.Automatic
        Drag.supportedActions: Qt.MoveAction
        Drag.hotSpot.x: width / 2
        Drag.hotSpot.y: height / 2
        Drag.mimeData: ({ [Settings.trayDragType]: icon.item ? String(icon.item.id) : "" })
    }

    MouseArea {
        id: area
        anchors.fill: parent
        hoverEnabled: true
        acceptedButtons: Qt.LeftButton | Qt.RightButton
        drag.target: dragProxy

        onPressed: function (event) {
            if (event.button === Qt.LeftButton)
                icon.grabToImage(function (result) { dragProxy.Drag.imageSource = result.url; },
                                  Qt.size(icon.width, icon.height));
        }
        onReleased: {
            dragProxy.x = 0;
            dragProxy.y = 0;
        }

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

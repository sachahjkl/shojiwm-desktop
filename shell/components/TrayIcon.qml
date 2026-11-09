import QtQuick
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
    /// Set from the drag itself: once the platform drag starts, the pointer
    /// belongs to it and the mouse area no longer sees it.
    property bool dragging: false
    /// The icon is on the taskbar: dragged off it and dropped where nothing
    /// takes it, it goes to the popover of hidden icons.
    property bool inTaskbar: false
    /// How far the icon stands aside while another is dragged over the
    /// taskbar, and whether it is the dragged one, shown faint where it would
    /// land.
    property real shift: 0
    property bool ghost: false

    opacity: icon.ghost ? 0.45 : 1
    transform: Translate {
        x: icon.shift
        Behavior on x {
            NumberAnimation { duration: Settings.motion(Theme.hoverIn); easing.type: Easing.OutCubic }
        }
    }

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
            NumberAnimation { duration: Settings.motion(Theme.fade) }
        }
    }

    ShellIcon {
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
        Drag.keys: [Settings.trayDragType]
        Drag.mimeData: ({ [Settings.trayDragType]: icon.item ? String(icon.item.id) : "" })
        Drag.onDragStarted: {
            icon.dragging = true;
            DragState.start("tray", icon.item ? String(icon.item.id) : "");
        }
        Drag.onDragFinished: function (dropAction) {
            icon.dragging = false;
            DragState.finish();
            if (icon.inTaskbar && dropAction === Qt.IgnoreAction && icon.item) {
                var id = String(icon.item.id);
                // After the drag has ended: the change removes this icon.
                Qt.callLater(function () { Settings.setTrayVisible(id, false); });
            }
            dragProxy.x = 0;
            dragProxy.y = 0;
        }
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

import QtQuick
import Quickshell.Bluetooth
import ".."

// The Bluetooth icon of the notification area: a rune on blue glass, grey
// while Bluetooth is off. It shows only when there is an adapter. A click
// opens the Bluetooth panel, a right click its menu (Taskbar.qml).
Item {
    id: icon

    /// The Bluetooth panel is open, so the icon stays lit.
    property bool active: false
    readonly property bool hovered: area.containsMouse
    signal clicked
    /// A right click; `at` is where the icon is in its window.
    signal menuRequested(rect at)

    readonly property var adapter: Bluetooth.defaultAdapter

    visible: icon.adapter !== null
    implicitWidth: visible ? Scheme.theme.trayIconSize + 8 : 0
    implicitHeight: Scheme.theme.taskbarHeight - Scheme.theme.taskbarPadding * 2

    Rectangle {
        anchors.fill: parent
        anchors.topMargin: 2
        anchors.bottomMargin: 2
        radius: Scheme.itemRadius
        color: "#ffffff"
        opacity: icon.active ? 0.22 : area.containsMouse ? 0.14 : 0
        Behavior on opacity {
            NumberAnimation { duration: Settings.motion(Scheme.theme.fade) }
        }
    }

    Image {
        anchors.centerIn: parent
        width: Scheme.theme.trayIconSize
        height: Scheme.theme.trayIconSize
        sourceSize: Qt.size(width * 2, height * 2)
        source: Qt.resolvedUrl("../assets/bluetooth"
            + (icon.adapter && icon.adapter.enabled ? "" : "-off") + ".svg")
    }

    MouseArea {
        id: area
        anchors.fill: parent
        hoverEnabled: true
        acceptedButtons: Qt.LeftButton | Qt.RightButton
        onClicked: function (event) {
            if (event.button === Qt.RightButton)
                icon.menuRequested(Qt.rect(icon.mapToItem(null, 0, 0).x, icon.mapToItem(null, 0, 0).y, icon.width, icon.height));
            else
                icon.clicked();
        }
    }
}

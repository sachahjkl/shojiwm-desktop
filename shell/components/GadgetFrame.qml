import QtQuick
import Quickshell
import ".."

// A desktop gadget's place: it holds one gadget, keeps its distance from the
// right and top edges of the desktop in a setting, and a drag anywhere on it
// moves it. Its buttons, if any, show on its right edge under the pointer,
// as Windows 7's gadget toolbar does.
Loader {
    id: frame

    /// The name of the setting that keeps the place, { right, top }.
    property string placeSetting: ""
    readonly property var place: Settings[frame.placeSetting] || { right: 24, top: 24 }
    /// The buttons: { icon (a theme icon name), run }.
    property var actions: []

    x: parent ? parent.width - width - frame.place.right : 0
    y: frame.place.top

    MouseArea {
        anchors.fill: parent
        z: 10
        cursorShape: pressed ? Qt.ClosedHandCursor : Qt.ArrowCursor
        property point grab
        onPressed: function (event) { grab = Qt.point(event.x, event.y); }
        onPositionChanged: function (event) {
            if (!pressed)
                return;
            var area = frame.parent;
            var at = mapToItem(area, event.x, event.y);
            Settings[frame.placeSetting] = {
                right: Math.round(Math.max(0, Math.min(area.width - frame.width,
                    area.width - (at.x - grab.x) - frame.width))),
                top: Math.round(Math.max(0, Math.min(area.height - frame.height, at.y - grab.y)))
            };
        }
    }

    HoverHandler {
        id: hover
    }

    Column {
        anchors.right: parent.right
        anchors.rightMargin: 4
        anchors.top: parent.top
        anchors.topMargin: 4
        z: 11
        spacing: 2
        visible: hover.hovered && frame.actions.length > 0

        Repeater {
            model: frame.actions

            delegate: Rectangle {
                id: button

                required property var modelData

                width: 20
                height: 20
                radius: 3
                color: buttonArea.containsMouse ? "#66ffffff" : "#33000000"
                border.width: 1
                border.color: "#80ffffff"

                ShellIcon {
                    anchors.centerIn: parent
                    implicitSize: 14
                    source: Quickshell.iconPath(button.modelData.icon, true)
                }

                MouseArea {
                    id: buttonArea
                    anchors.fill: parent
                    hoverEnabled: true
                    onClicked: button.modelData.run()
                }
            }
        }
    }
}

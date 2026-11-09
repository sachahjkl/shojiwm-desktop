import QtQuick
import ".."

// A desktop gadget's place: it holds one gadget, keeps its distance from the
// right and top edges of the desktop in a setting, and a drag anywhere on it
// moves it.
Loader {
    id: frame

    /// The name of the setting that keeps the place, { right, top }.
    property string placeSetting: ""
    readonly property var place: Settings[frame.placeSetting] || { right: 24, top: 24 }

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
}

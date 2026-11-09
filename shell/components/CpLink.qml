import QtQuick
import ".."
import "../theme.js" as Theme

// A link of the Control Panel: blue, underlined under the pointer.
Text {
    id: link

    property bool large: false
    signal clicked

    color: area.containsMouse ? (Scheme.dark ? "#a8d4ff" : "#3399ff") : Scheme.link2
    font.family: Theme.fontFamily
    font.pixelSize: large ? Theme.fontSize + 3 : Theme.fontSize
    font.underline: area.containsMouse
    elide: Text.ElideRight

    // The text only, not the empty width beside a short link.
    MouseArea {
        id: area
        width: link.contentWidth
        height: link.contentHeight
        hoverEnabled: true
        cursorShape: Qt.PointingHandCursor
        onClicked: link.clicked()
    }
}

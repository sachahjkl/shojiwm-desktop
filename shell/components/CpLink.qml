import QtQuick
import ".."

// A link of the Control Panel: blue, underlined under the pointer.
Text {
    id: link

    property bool large: false
    signal clicked

    color: area.containsMouse ? (Scheme.dark ? "#a8d4ff" : "#3399ff") : Scheme.link2
    font.family: Scheme.theme.fontFamily
    font.pixelSize: large ? Scheme.theme.fontSize + 3 : Scheme.theme.fontSize
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

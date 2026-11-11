import QtQuick
import ".."

// A month arrow of the calendar.
Rectangle {
    id: arrow

    property string glyph: ""
    signal activated

    width: 24
    height: 22
    radius: Scheme.itemRadius
    color: area.containsMouse ? Scheme.hover : "transparent"
    border.width: area.containsMouse ? 1 : 0
    border.color: Scheme.selectionBorder

    Text {
        anchors.centerIn: parent
        text: arrow.glyph
        color: Scheme.text
        font.family: Scheme.theme.fontFamily
        font.pixelSize: Scheme.theme.fontSize
    }

    MouseArea {
        id: area
        anchors.fill: parent
        hoverEnabled: true
        onClicked: arrow.activated()
    }
}

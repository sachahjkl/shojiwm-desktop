import QtQuick
import ".."
import "../theme.js" as Theme

// A month arrow of the calendar.
Rectangle {
    id: arrow

    property string glyph: ""
    signal activated

    width: 24
    height: 22
    radius: 3
    color: area.containsMouse ? Scheme.hover : "transparent"
    border.width: area.containsMouse ? 1 : 0
    border.color: Scheme.selectionBorder

    Text {
        anchors.centerIn: parent
        text: arrow.glyph
        color: Scheme.text
        font.family: Theme.fontFamily
        font.pixelSize: Theme.fontSize
    }

    MouseArea {
        id: area
        anchors.fill: parent
        hoverEnabled: true
        onClicked: arrow.activated()
    }
}

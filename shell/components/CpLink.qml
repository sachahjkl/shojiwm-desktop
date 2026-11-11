import QtQuick
import ".."

// A link of the Control Panel: blue, underlined under the pointer.
Text {
    id: link

    property bool large: false
    /// Its colour at rest; lighter under the pointer. A Kernel category gives its own.
    property color tint: Scheme.link2
    signal clicked

    color: !area.containsMouse ? link.tint
        : Scheme.kernel ? Qt.lighter(link.tint, 1.35)
        : Scheme.dark ? "#a8d4ff" : "#3399ff"
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

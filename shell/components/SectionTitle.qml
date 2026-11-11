import QtQuick
import ".."

// The title of a part of a popup or a page, drawn as the theme says. Aero: the
// words alone, in the heading colour. Kernel: a short bar in the accent colour,
// then the words in spaced, monospaced capitals.
Item {
    id: title

    property string text: ""
    property int pixelSize: Scheme.theme.fontSizeSmall
    property color color: Scheme.menuHeading
    /// The bar's colour (Kernel): the theme's accent unless a category gives its own.
    property color accent: Scheme.theme.calendarToday
    property real topPadding: 0

    implicitWidth: label.x + label.implicitWidth
    implicitHeight: topPadding + label.implicitHeight

    Rectangle {
        visible: Scheme.theme.headingBar > 0
        x: 0
        y: title.topPadding + Math.round((label.implicitHeight - height) / 2)
        width: Scheme.theme.headingBar
        height: Math.round(label.font.pixelSize * 0.95)
        color: title.accent
    }

    Text {
        id: label
        x: Scheme.theme.headingBar > 0 ? Scheme.theme.headingBar + 6 : 0
        y: title.topPadding
        width: title.width > 0 ? Math.max(0, title.width - x) : implicitWidth
        text: title.text
        color: title.color
        elide: Text.ElideRight
        font.family: Scheme.theme.headingFamily
        font.pixelSize: title.pixelSize
        font.capitalization: Scheme.theme.headingCaps ? Font.AllUppercase : Font.MixedCase
        font.letterSpacing: Scheme.theme.headingSpacing
    }
}

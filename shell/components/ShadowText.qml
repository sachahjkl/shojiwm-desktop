import QtQuick
import "../theme.js" as Theme

// White text with the soft dark shadow Windows 7 puts under the text of the
// log-on screen, so it reads on any picture.
Item {
    id: shadowed

    property alias text: label.text
    property alias font: label.font

    implicitWidth: label.implicitWidth
    implicitHeight: label.implicitHeight

    // The shadow: the text again, dark, a pixel down, then once more wider and
    // fainter. Text has no blur of its own.
    Text {
        x: 0
        y: 1
        text: label.text
        font: label.font
        color: "#99000000"
    }
    Text {
        x: 1
        y: 2
        text: label.text
        font: label.font
        color: "#33000000"
    }

    Text {
        id: label
        color: "#ffffff"
        font.family: Theme.fontFamily
    }
}

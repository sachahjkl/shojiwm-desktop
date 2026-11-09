import QtQuick
import QtQuick.Window

// A theme icon, sharp at any scale. Quickshell's IconImage asks the icon
// theme for its size in logical pixels: on a scaled screen, a theme that
// has fixed PNG sizes (Steam's: 16 to 48, then 256) gives a smaller
// picture, which the scale enlarges and blurs. This one asks for the size
// in the screen's own pixels. Used as IconImage is: `implicitSize` and
// `source` (Quickshell.iconPath).
Item {
    id: icon

    property real implicitSize: 0
    property alias source: image.source

    implicitWidth: icon.implicitSize
    implicitHeight: icon.implicitSize

    Image {
        id: image

        anchors.fill: parent
        fillMode: Image.PreserveAspectFit
        smooth: true
        mipmap: true
        sourceSize.width: Math.ceil(icon.width * Screen.devicePixelRatio)
        sourceSize.height: Math.ceil(icon.height * Screen.devicePixelRatio)
    }
}

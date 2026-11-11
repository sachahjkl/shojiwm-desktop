pragma ComponentBehavior: Bound

import QtQuick
import Quickshell
import ".."

// The Action Center panel: the problems the system has now, each with what
// can be done about it, or a word that all is well.
PopupWindow {
    id: popup

    property Item anchorItem: null
    property bool open: false

    signal dismissed

    color: "transparent"
    visible: popup.open && popup.anchorItem !== null

    onVisibleChanged: {
        if (visible)
            Maintenance.refresh();
    }

    anchor.item: popup.anchorItem
    anchor.rect: Qt.rect(0, 0,
                         popup.anchorItem ? popup.anchorItem.width : 1,
                         popup.anchorItem ? popup.anchorItem.height : 1)
    anchor.edges: Scheme.awayFromBar
    anchor.gravity: Scheme.awayFromBar
    anchor.margins.bottom: Scheme.popupMargin(Scheme.theme.popupGap + 2)
    anchor.adjustment: PopupAdjustment.SlideX | PopupAdjustment.FlipY

    implicitWidth: 320
    implicitHeight: body.implicitHeight + 20

    Rectangle {
        anchors.fill: parent
        radius: Scheme.popupRadius
        border.width: 1
        border.color: Scheme.popupOutline
        gradient: Gradient {
            GradientStop { position: 0.0; color: Scheme.popupTop }
            GradientStop { position: 1.0; color: Scheme.popupBottom }
        }
    }

    Rectangle {
        anchors.fill: parent
        anchors.margins: 1
        radius: Scheme.corner(Math.max(0, Scheme.popupRadius - 1))
        color: "transparent"
        border.width: 1
        border.color: Scheme.popupInnerLine
    }

    Column {
        id: body

        x: 10
        y: 10
        width: parent.width - 20
        spacing: 8

        Text {
            text: Maintenance.issues.length === 0 ? "Action Center"
                : Maintenance.issues.length + (Maintenance.issues.length === 1 ? " message" : " messages")
            color: Scheme.menuHeading
            font.family: Scheme.theme.fontFamily
            font.pixelSize: Scheme.theme.fontSize
        }

        Text {
            visible: Maintenance.issues.length === 0
            width: parent.width
            text: "No current issues detected: every service runs, and the disks have room."
            color: Scheme.text
            wrapMode: Text.WordWrap
            font.family: Scheme.theme.fontFamily
            font.pixelSize: Scheme.theme.fontSizeSmall
        }

        Repeater {
            model: Maintenance.issues

            delegate: Rectangle {
                id: issue

                required property var modelData

                width: body.width
                height: column.implicitHeight + 14
                radius: Scheme.itemRadius
                color: issue.modelData.level === "error"
                    ? (Scheme.dark ? "#33d9302a" : "#1fd9302a")
                    : (Scheme.dark ? "#33f2c335" : "#26f2c335")
                border.width: 1
                border.color: issue.modelData.level === "error" ? "#99d9302a" : "#99c9a020"

                Image {
                    x: 8
                    y: 8
                    width: 16
                    height: 16
                    sourceSize: Qt.size(32, 32)
                    source: Qt.resolvedUrl("../assets/network-badge-"
                        + (issue.modelData.level === "error" ? "error" : "warning") + ".svg")
                }

                Column {
                    id: column

                    x: 32
                    y: 7
                    width: parent.width - 40
                    spacing: 3

                    Text {
                        width: parent.width
                        text: issue.modelData.title
                        color: Scheme.text
                        font.family: Scheme.theme.fontFamily
                        font.pixelSize: Scheme.theme.fontSize
                        font.bold: true
                        elide: Text.ElideRight
                    }

                    Text {
                        width: parent.width
                        text: issue.modelData.detail
                        color: Scheme.textMuted
                        wrapMode: Text.WordWrap
                        font.family: Scheme.theme.fontFamily
                        font.pixelSize: Scheme.theme.fontSizeSmall
                    }

                    Row {
                        spacing: 12

                        CpLink {
                            visible: issue.modelData.unit.length > 0 && issue.modelData.user
                            text: "Clear this message"
                            onClicked: Maintenance.resetUnit(issue.modelData)
                        }

                        Text {
                            visible: issue.modelData.unit.length > 0 && !issue.modelData.user
                            text: "journalctl -u " + issue.modelData.unit
                            color: Scheme.textMuted
                            font.family: "monospace"
                            font.pixelSize: Scheme.theme.fontSizeSmall
                        }

                        CpLink {
                            visible: issue.modelData.mount.length > 0
                            text: "Open the disk"
                            onClicked: Maintenance.openMount(issue.modelData)
                        }
                    }
                }
            }
        }
    }
}

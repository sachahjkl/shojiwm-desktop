pragma ComponentBehavior: Bound

import QtQuick
import Quickshell
import ".."
import "../theme.js" as Theme

// The notification panel a click on the bell opens: the notifications of
// this session, newest first, each with its program's icon, its title, its
// text and when it came. A cross removes one, Clear all removes them all,
// and the switch at the bottom is Hokum's Do not disturb.
PopupWindow {
    id: popup

    property Item anchorItem: null
    property bool open: false

    signal dismissed

    /// The time the ages count from, renewed while the panel is open.
    property real now: Date.now()

    color: "transparent"
    visible: popup.open && popup.anchorItem !== null

    onVisibleChanged: {
        popup.heldHeight = visible ? popup.naturalHeight : 0;
        if (visible) {
            popup.now = Date.now();
            NotificationHistory.markRead();
        }
    }

    Timer {
        interval: 30000
        repeat: true
        running: popup.visible
        onTriggered: popup.now = Date.now()
    }

    anchor.item: popup.anchorItem
    anchor.rect: Qt.rect(0, 0,
                         popup.anchorItem ? popup.anchorItem.width : 1,
                         popup.anchorItem ? popup.anchorItem.height : 1)
    anchor.edges: Edges.Top
    anchor.gravity: Edges.Top
    anchor.margins.bottom: Theme.popupGap + 2
    anchor.adjustment: PopupAdjustment.SlideX | PopupAdjustment.FlipY

    implicitWidth: 340
    // The panel grows above the bell, so a shorter list would move its top
    // edge down from under the pointer, which closes it. While it is open,
    // it keeps the tallest height it had; the list keeps its place at the
    // top.
    readonly property real naturalHeight: header.height + list.height + empty.height + footer.height + 20
    property real heldHeight: 0
    onNaturalHeightChanged: {
        if (popup.visible)
            popup.heldHeight = Math.max(popup.heldHeight, popup.naturalHeight);
    }
    implicitHeight: Math.max(popup.naturalHeight, popup.heldHeight)

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

    Item {
        id: header

        x: 10
        y: 8
        width: parent.width - 20
        height: 22

        Text {
            anchors.verticalCenter: parent.verticalCenter
            text: "Notifications"
            color: Scheme.menuHeading
            font.family: Theme.fontFamily
            font.pixelSize: Theme.fontSize
        }

        Text {
            anchors.right: parent.right
            anchors.verticalCenter: parent.verticalCenter
            visible: NotificationHistory.entries.length > 0
            text: "Clear all"
            color: Scheme.link
            font.family: Theme.fontFamily
            font.pixelSize: Theme.fontSizeSmall
            font.underline: clearArea.containsMouse

            MouseArea {
                id: clearArea
                anchors.fill: parent
                anchors.margins: -3
                hoverEnabled: true
                cursorShape: Qt.PointingHandCursor
                onClicked: {
                    NotificationHistory.clear();
                }
            }
        }
    }

    ListView {
        id: list

        x: 6
        y: header.y + header.height + 4
        width: parent.width - 12
        height: Math.min(contentHeight, 380)
        clip: true
        spacing: 2
        boundsBehavior: Flickable.StopAtBounds
        model: NotificationHistory.entries

        delegate: Rectangle {
            id: card

            required property var modelData
            required property int index

            width: list.width
            height: Math.max(44, text.implicitHeight + 14)
            radius: Scheme.itemRadius
            color: cardArea.containsMouse ? Scheme.hover : "transparent"
            border.width: cardArea.containsMouse ? 1 : 0
            border.color: Scheme.selectionBorder

            MouseArea {
                id: cardArea
                anchors.fill: parent
                hoverEnabled: true
            }

            ShellIcon {
                x: 8
                y: 8
                implicitSize: 24
                source: Quickshell.iconPath(card.modelData.icon.length > 0 ? card.modelData.icon
                    : card.modelData.app.toLowerCase(), "dialog-information")
            }

            Column {
                id: text

                x: 40
                y: 7
                width: parent.width - 40 - 28
                spacing: 1

                Row {
                    width: parent.width
                    spacing: 6

                    Text {
                        width: parent.width - age.implicitWidth - 6
                        text: card.modelData.summary.length > 0 ? card.modelData.summary : card.modelData.app
                        color: Scheme.text
                        font.family: Theme.fontFamily
                        font.pixelSize: Theme.fontSize
                        font.bold: true
                        elide: Text.ElideRight
                    }

                    Text {
                        id: age
                        text: NotificationHistory.age(card.modelData.time, popup.now)
                        color: Scheme.textMuted
                        font.family: Theme.fontFamily
                        font.pixelSize: Theme.fontSizeSmall
                    }
                }

                Text {
                    width: parent.width
                    visible: card.modelData.body.length > 0
                    text: card.modelData.body
                    color: Scheme.text
                    wrapMode: Text.Wrap
                    maximumLineCount: 3
                    elide: Text.ElideRight
                    font.family: Theme.fontFamily
                    font.pixelSize: Theme.fontSizeSmall
                }

                Text {
                    text: card.modelData.app
                    visible: card.modelData.summary.length > 0 && card.modelData.app.length > 0
                    color: Scheme.textMuted
                    font.family: Theme.fontFamily
                    font.pixelSize: Theme.fontSizeSmall - 1
                }
            }

            // The cross that removes this one, under the pointer.
            Text {
                anchors.right: parent.right
                anchors.rightMargin: 8
                y: 6
                visible: cardArea.containsMouse || crossArea.containsMouse
                text: "×"
                color: crossArea.containsMouse ? "#d9302a" : Scheme.textMuted
                font.pixelSize: 16

                MouseArea {
                    id: crossArea
                    anchors.fill: parent
                    anchors.margins: -4
                    hoverEnabled: true
                    onClicked: {
                        NotificationHistory.remove(card.index);
                    }
                }
            }
        }
    }

    Text {
        id: empty

        x: 16
        y: list.y
        height: NotificationHistory.entries.length === 0 ? 40 : 0
        visible: height > 0
        verticalAlignment: Text.AlignVCenter
        text: "No new notifications."
        color: Scheme.textMuted
        font.family: Theme.fontFamily
        font.pixelSize: Theme.fontSize
    }

    // Do not disturb, on the band at the bottom.
    Rectangle {
        id: footer

        anchors.left: parent.left
        anchors.right: parent.right
        anchors.bottom: parent.bottom
        anchors.margins: 1
        height: 32
        radius: Scheme.corner(Math.max(0, Scheme.popupRadius - 1))
        color: Scheme.dark ? "#33000000" : "#80ffffff"

        Rectangle {
            anchors.top: parent.top
            anchors.left: parent.left
            anchors.right: parent.right
            height: 1
            color: Scheme.menuDivider
        }

        CpCheck {
            anchors.left: parent.left
            anchors.leftMargin: 10
            anchors.verticalCenter: parent.verticalCenter
            label: "Do not disturb"
            checked: NotificationHistory.doNotDisturb
            onToggled: function (on) { NotificationHistory.setDoNotDisturb(on); }
        }
    }
}

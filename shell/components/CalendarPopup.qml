pragma ComponentBehavior: Bound

import QtQuick
import Quickshell
import ".."

// The calendar a click on the clock opens, as Windows 7 shows it: the long
// date at the top, then a month with today framed, and arrows for the months
// around it. Weeks start on the locale's first day.
PopupWindow {
    id: popup

    property Item anchorItem: null
    property bool open: false

    signal dismissed

    /// The first day of the month on show.
    property date month: popup.firstOfMonth(new Date())

    readonly property var locale: Qt.locale()
    readonly property date today: clock.date

    SystemClock {
        id: clock
        precision: SystemClock.Minutes
    }

    // Opening always starts from this month.
    onVisibleChanged: {
        if (visible)
            popup.month = popup.firstOfMonth(new Date());
    }

    function firstOfMonth(day) {
        return new Date(day.getFullYear(), day.getMonth(), 1);
    }

    function shift(months) {
        popup.month = new Date(popup.month.getFullYear(),
                               popup.month.getMonth() + months, 1);
    }

    function sameDay(a, b) {
        return a.getFullYear() === b.getFullYear()
            && a.getMonth() === b.getMonth() && a.getDate() === b.getDate();
    }

    /// The 42 days of the grid: the month, padded with its neighbours.
    readonly property var days: {
        // Qt counts Sunday as 7, JavaScript as 0.
        var firstWeekday = popup.locale.firstDayOfWeek % 7;
        var lead = (popup.month.getDay() - firstWeekday + 7) % 7;
        var list = [];
        for (var i = 0; i < 42; i++) {
            list.push(new Date(popup.month.getFullYear(), popup.month.getMonth(),
                               1 - lead + i));
        }
        return list;
    }

    readonly property var weekdays: {
        var first = popup.locale.firstDayOfWeek;
        var names = [];
        for (var i = 0; i < 7; i++) {
            // dayName takes 0 for Sunday, as JavaScript does.
            names.push(popup.locale.dayName((first + i) % 7, Locale.ShortFormat));
        }
        return names;
    }

    readonly property int cell: 30

    color: "transparent"
    visible: popup.open && popup.anchorItem !== null

    anchor.item: popup.anchorItem
    anchor.rect: Qt.rect(0, 0,
                         popup.anchorItem ? popup.anchorItem.width : 1,
                         popup.anchorItem ? popup.anchorItem.height : 1)
    anchor.edges: Edges.Top | Edges.Right
    anchor.gravity: Edges.Top | Edges.Left
    anchor.margins.bottom: Scheme.theme.popupGap + 2
    anchor.adjustment: PopupAdjustment.SlideX | PopupAdjustment.FlipY

    implicitWidth: popup.cell * 7 + 24
    implicitHeight: body.implicitHeight + 24

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

    // The light line inside the outline, as every Windows 7 menu has.
    Rectangle {
        anchors.fill: parent
        anchors.margins: 1
        radius: Scheme.corner(Math.max(0, Scheme.popupRadius - 1))
        color: "transparent"
        border.width: 1
        border.color: Scheme.popupInnerLine
    }

    // A hover handler, not a MouseArea: it sees the pointer over the rows
    // and icons too, which take the hover from a MouseArea under them.

    Column {
        id: body

        anchors.left: parent.left
        anchors.right: parent.right
        anchors.top: parent.top
        anchors.margins: 12
        spacing: 6

        // The long date, in the link blue Windows uses there.
        Text {
            width: parent.width
            horizontalAlignment: Text.AlignHCenter
            text: popup.today.toLocaleDateString(popup.locale, Locale.LongFormat)
            color: Scheme.link
            font.family: Scheme.theme.fontFamily
            font.pixelSize: Scheme.theme.fontSize
            elide: Text.ElideRight
        }

        Rectangle {
            width: parent.width
            height: 1
            color: Scheme.menuDivider
        }

        // The month, between its arrows.
        Item {
            width: parent.width
            height: 26

            CalendarArrow {
                anchors.left: parent.left
                anchors.verticalCenter: parent.verticalCenter
                glyph: "◂"
                onActivated: popup.shift(-1)
            }

            Text {
                anchors.centerIn: parent
                text: popup.month.toLocaleDateString(popup.locale, "MMMM yyyy")
                color: Scheme.text
                font.family: Scheme.theme.fontFamily
                font.pixelSize: Scheme.theme.fontSize + 1
                font.bold: true
            }

            CalendarArrow {
                anchors.right: parent.right
                anchors.verticalCenter: parent.verticalCenter
                glyph: "▸"
                onActivated: popup.shift(1)
            }
        }

        Grid {
            columns: 7

            Repeater {
                model: popup.weekdays

                delegate: Text {
                    required property var modelData
                    width: popup.cell
                    height: 20
                    horizontalAlignment: Text.AlignHCenter
                    text: modelData
                    color: Scheme.menuHeading
                    font.family: Scheme.theme.fontFamily
                    font.pixelSize: Scheme.theme.fontSizeSmall
                }
            }

            Repeater {
                model: popup.days

                delegate: Item {
                    id: day

                    required property var modelData
                    readonly property bool inMonth:
                        modelData.getMonth() === popup.month.getMonth()
                    readonly property bool isToday: popup.sameDay(modelData, popup.today)

                    width: popup.cell
                    height: 24

                    Rectangle {
                        anchors.fill: parent
                        anchors.margins: 1
                        radius: Scheme.corner(2)
                        color: day.isToday ? Scheme.selection
                            : dayArea.containsMouse ? Scheme.hover : "transparent"
                        border.width: day.isToday || dayArea.containsMouse ? 1 : 0
                        border.color: day.isToday ? Scheme.today
                            : Scheme.selectionBorder
                    }

                    Text {
                        anchors.centerIn: parent
                        text: day.modelData.getDate()
                        color: day.inMonth ? Scheme.text : Scheme.otherMonth
                        font.family: Scheme.theme.fontFamily
                        font.pixelSize: Scheme.theme.fontSize
                        font.bold: day.isToday
                    }

                    MouseArea {
                        id: dayArea
                        anchors.fill: parent
                        hoverEnabled: true
                        // A day of the next or the last month turns to it.
                        onClicked: {
                            if (!day.inMonth)
                                popup.month = popup.firstOfMonth(day.modelData);
                        }
                    }
                }
            }
        }

        // Back to today, when another month is on show.
        Text {
            width: parent.width
            height: visible ? implicitHeight : 0
            horizontalAlignment: Text.AlignHCenter
            visible: !popup.sameDay(popup.month, popup.firstOfMonth(popup.today))
            text: "Today"
            color: Scheme.link
            font.family: Scheme.theme.fontFamily
            font.pixelSize: Scheme.theme.fontSize
            font.underline: todayArea.containsMouse

            MouseArea {
                id: todayArea
                anchors.fill: parent
                hoverEnabled: true
                cursorShape: Qt.PointingHandCursor
                onClicked: popup.month = popup.firstOfMonth(new Date())
            }
        }

        // The additional clocks, as Windows 7 lists them under the month.
        Rectangle {
            width: parent.width
            height: 1
            color: Scheme.menuDivider
            visible: WorldClocks.clocks.length > 0
        }

        Repeater {
            model: WorldClocks.clocks

            delegate: Item {
                id: worldClock

                required property var modelData

                width: body.width
                height: 34

                Column {
                    anchors.left: parent.left
                    anchors.verticalCenter: parent.verticalCenter

                    Text {
                        text: worldClock.modelData.label
                        color: Scheme.text
                        font.family: Scheme.theme.fontFamily
                        font.pixelSize: Scheme.theme.fontSize
                    }

                    Text {
                        text: worldClock.modelData.day
                        color: Scheme.textMuted
                        font.family: Scheme.theme.fontFamily
                        font.pixelSize: Scheme.theme.fontSizeSmall
                    }
                }

                Text {
                    anchors.right: parent.right
                    anchors.verticalCenter: parent.verticalCenter
                    text: worldClock.modelData.time
                    color: Scheme.text
                    font.family: Scheme.theme.fontFamily
                    font.pixelSize: Scheme.theme.fontSize + 4
                    font.weight: Font.Light
                }
            }
        }
    }
}

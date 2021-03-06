# Ksysguard-Sensord #

<!-- TODO: Badges for coverage and test status -->

A ksysguardd-alike that exposes aribtrary configured sensor data to a querying ksysguard instance.
The protocol for the communication is described in: https://invent.kde.org/plasma/ksysguard/-/blob/master/ksysguardd/Porting-HOWTO

<!-- 
This document describes the interface between ksysguard and
ksysguardd. ksysguardd is started as a child of ksysguard, either
directly or via a shell. Alternatively a ksysguardd can listen on a
port and a single instance can then be used by multiple instances of
ksysguard.

This client/server design was chosen, because on some operating
systems the back-end needs elevated permissions.
It also allowed for an easy network support using existing
security mechanisms (ssh).

ksysguard sends commands and ksysguardd answers to them. Each answer
ends with the string "\nksysguardd> ". Error messages are enclosed in
ESC '\033' characters. Therefor regular messages may never contain
ESC. The set of commands that each ksysguardd implementation supports
can be very different. There are only a very few mandatory command and
a few recommended commands.

The mandatory commands are 'monitors', 'test' and 'quit'.
The recommended commands are:

cpu/system/idle
cpu/system/sys
cpu/system/nice
cpu/system/user
mem/swap/free
mem/swap/used
mem/physical/cached
mem/physical/buf
mem/physical/application
mem/physical/used
mem/physical/free
ps
pscount

Without these commands KSysGuard is not very helpful.

The 'monitors' command returns the list of available sensors. The
output looks like this:

--------
mem/physical/free       integer
ps      table
pscount integer
ksysguardd> 
--------

Sensor names can be hierarchical. Each level is separated by a
/. If you don't want a '/' character to be interpreted as a hierarchy
delimiter, you can escape it with a preceding '\'. ksysguard uses a tree
widget in the SensorBrowser to display the commands in a tree. Every
sensor name must be followed by the type of the sensor separated by a
tab. Currently 4 different types of sensors are supported, integer,
float, listview and table. The table sensor returns the information for
the ProcessController widget. listview sensors use a generic table to
display information. To find out more about a sensor an additional
command must be implemented for each sensor that has a questionmark
appended to the sensor name. It can be used to find out more about the
sensor.

--------
ksysguardd> mem/physical/free?
Free Memory     0       260708  KB
ksysguardd>
--------

integer and float sensor return a short description, the minimum and
maximum value and the unit. All fields are again separated by
tabs. The minimum and maximum values can both be 0 to trigger the
auto-range feature of the display.

--------
ksysguardd> ps?
Name    PID     PPID    UID     GID     Status  User%   System% Nice    VmSize  VmRss   Login   TracerPID       Command
s       d       d       d       d       S       f       f       d       D       D       s       d       s
ksysguardd>
--------

This is the output of the ps? inquiry command. The ps command is the
only recommended command. The answer to ps? consists of 2 lines. Both
lines have the same number of fields each separated by a tab. The
first line specifies the name of the columns and the second the type
of the values in the column.

d: integer value
D: integer value that should be localized in the frontend
f: floating point value
s: string value
S: string value that needs to be translated
   Strings must be added to the ProcessList::columnDict dictionary.

For the ProcessController to function properly the Name and PID
columns are mandatory. All other columns are optional and their
content may be implementation dependant. It is highly recommended not
to deviate from the Linux implementation unless absolutely
unavoidable, in order to provide a consistent interface across all
platforms.

The 'test' command can be used by the front-end to find out if a
certain other command is supported by this version of ksysguardd. The
command returns "1\n" if the command is supported and "0\n" if the
command is not supported.

The 'quit' command terminates ksysguardd.

ksysguardd may support dynamic monitor sets. If a CPU is added or an
interface disabled, monitors may be added or removed. To notify the
front-end about this, you need to send the string "RECONFIGURE" over
stderr. This causes the front-end to request the list of monitors
again and adapt it's layout accordingly. If ksysguardd receives a
command it doesn't know it has to reply with "UNKNOWN
COMMAND\nksysguardd> ".


-->

### Installation ###

The installation should be straightforward for most systems that include systemd.
Installing the compiled binary to `/usr/bin` as well as installing and enabling the systemd unit is sufficient.

The configuration of the sensors exposed by the daemon is expected in `/etc/ksysguard-sensord/config.d/`.
The daemon itself is configured in `/etc/ksysguard-sensord/config`.

### Configuration ###

TBD
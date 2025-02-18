# Timesheets

A simple timesheets language for tracking surplus or deficit of worked time.

## Example

Assume the file `my-timesheet.tsh` contains the following snippet:
```
WEEK 16th September 2024
  Monday
    WORKING DAY 09:00 - 19:09
    BREAK 09:06 - 09:12 | First break
    BREAK 10:45 - 11:00 | Second break
    LUNCH 12:38 - 13:42

  Tuesday
    WORKING DAY 09:07 - 17:43
    BREAK 10m | Short break
    LUNCH 12:25 - 13:27

  Wednesday
    WORKING DAY 09:03 - 17:52
    LUNCH 11:45 - 12:56

  Thursday
    WORKING DAY 09:06 - 19:25
    LUNCH 12:01 - 13:02

  Friday
    LEAVE 7h 30m | Annual leave

WEEK 23rd September 2024
  Monday
    WORKING DAY 09:00 - 17:45
    LUNCH 100m | Very hungry

  Tuesday
    WORKING DAY 08:48 - NOW

```

Running the program (or generated binary) on the file  
`cargo r -- my-timesheet.tsh`  
at 12:00 on Tuesday will produce the following output:
```
    ┌───────┐
    │ +3:04 │ Week starting 16th September 2024
    ├───────┤
    │ +1:14 │ Monday
    │ -6m   │ Tuesday
    │ +8m   │ Wednesday
    │ +1:48 │ Thursday
    │ +0m   │ Friday
    └───────┘
    ┌───────┐
    │ -4:42 │ Week starting 23rd September 2024
    ├───────┤
    │ -25m  │ Monday
    │ -4:17 │ Tuesday
    └───────┘
    ┌───────┐
    │ +2:39 │ TOTAL CREDIT BEFORE TODAY
    │ -1:38 │ TOTAL DEFICIT NOW
    └───────┘
    ┌───────┐
    │ 13:39 │ EARLIEST FINISH TIME
    │ 14:39 │ EARLIEST FINISH TIME + LUNCH
    ├───────┤
    │ 16:18 │ RETAIN CREDIT
    │ 17:18 │ RETAIN CREDIT + LUNCH
    └───────┘
```

## Timesheets syntax

A file is broken into weeks, which are broken into days.
A day can be `Monday` to `Sunday` and can contain any number of logs.  

A log takes on the following shape:  
```
    LOG_EVENT PERIOD | Commentary
```

Where a log event can take one of the following forms
|Log event|Effect on work surplus|Must include commentary?|
|---|---|---|
|`WORKING DAY`|Credit|No|
|`WORK`|Credit|Yes|
|`LUNCH`|Debit|No|
|`LEAVE`|Credit|No|
|`BREAK`|Debit|Yes|

While a period can take the following forms
```
##m
##h ##m
##:## - ##:##
##:## - NOW
```
The `| Commentary` clause is optional for standard log events, but is necessary for `WORK` and `BREAK` logs as these represent events that distort the standard working day.


## Install
`cargo build`
Depends on `lang_packer`


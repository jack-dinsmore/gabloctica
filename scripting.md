# Command list

||||||||
|-|-|-|-|-|-|-|
|**Stack manipulation**|push|pop|dup|popn|dupn|swp|
|**Control flow**|jmp|jnz|irp|puship|popip|call|
|**Comparison**|lt|gt|le|ge|eq|
|**Math**|add|sub|mul|div|neg|pow|
|**Boolean operations**|and|or|xor|not|
|**Timing operations**|nop|tick|

Most of these operations take no arguments. The exceptions are push (pushes a literal or label), jmp and jnz (jumps to the label), and call (says the number of arguments to push).

# Functions

|Index|No. args|No. returns|Description|
|-|-|-|-|
|0|1|0|Print #1|
|1|3|0|Add a force (#1, #2, #3) to the host object|
|2|3|0|Add a torque (#1, #2, #3) to the host object|
|3|2|0|Emit a signal to all blocks of class #1 on the signal network. The first #2 items on the stack are sent.|
|4|2|1|If a signal from class #1 is received on the attached signal network, jump to #2.|
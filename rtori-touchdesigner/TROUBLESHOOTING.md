# Troubleshooting notes while building this thing

When a dependent dynamic lib for a plugin cannot be found, no error is returned.
To debug that, set `DYLD_PRINT_LIBRARIES_POST_LAUNCH` to `1`, it will print the lirbaries loaded during runtime.
If there are some `Library not loaded` errors, then we know where the cause is.

In case of doubt, you can use all the following env variables to debug it:
```bash
export DYLD_PRINT_APIS=1
export DYLD_PRINT_ENV=1
export DYLD_PRINT_OPS=1
export DYLD_PRINT_INITIALIZERS=1
export DYLD_PRINT_LIBRARIES=1
export DYLD_PRINT_LIBRARIES_POST_LAUNCH=1
export DYLD_PRINT_SEGMENTS=1
export DYLD_PRINT_STATISTICS=1
```
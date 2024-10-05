#define RTORI_ALLOC_SIZED(size_argument_index)                                 \
  __attribute__(alloc_size(size_argument_index))

/// to be used when a pointer could be string-like, but isn't
/// https://gcc.gnu.org/onlinedocs/gcc/Common-Variable-Attributes.html#index-nonstring-variable-attribute
#define RTORI_NONSTRING __attribute__(nonstring)

/// Also defines an input as being nonnull
#define RTORI_SLICE_RO(ptr, len)

/// Also defines an input as being nonnull
#define RTORI_SLICE_RW(ptr, len)

/// Also defines an input as being nonnull
#define RTORI_SLICE_WO(ptr, len)
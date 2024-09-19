namespace rtori_core
{
    [DllImport(
        "rtori_core",
        EntryPoint = "rtori_init")]
    internal static extern IntPtr Init();

}
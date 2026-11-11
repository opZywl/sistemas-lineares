using System;
using System.Collections.Generic;
using System.Drawing;
using System.Drawing.Text;
using System.IO;
using System.Reflection;
using System.Runtime.InteropServices;

namespace SistemasLineares {
    internal static class EmbeddedFonts {
        private static readonly PrivateFontCollection collection=new PrivateFontCollection();
        // As fontes permanecem disponíveis durante toda a vida do processo.
        private static readonly List<IntPtr> buffers=new List<IntPtr>();
        private static readonly FontFamily family=Load();
        [DllImport("gdi32.dll")] private static extern IntPtr AddFontMemResourceEx(IntPtr data,uint length,IntPtr reserved,ref uint count);
        private static FontFamily Load() {
            try {
                foreach(string name in new[]{"Regular","Bold"}) {
                    using(Stream stream=Assembly.GetExecutingAssembly().GetManifestResourceStream("Fonts."+name)) {
                        if(stream==null) return FontFamily.GenericSansSerif;
                        byte[] bytes=new byte[stream.Length]; int offset=0, read;
                        while(offset<bytes.Length && (read=stream.Read(bytes,offset,bytes.Length-offset))>0) offset+=read;
                        IntPtr memory=Marshal.AllocCoTaskMem(bytes.Length);
                        buffers.Add(memory); Marshal.Copy(bytes,0,memory,bytes.Length); collection.AddMemoryFont(memory,bytes.Length); uint count=0; AddFontMemResourceEx(memory,(uint)bytes.Length,IntPtr.Zero,ref count);
                    }
                }
                return collection.Families[0];
            } catch(ArgumentException) { return FontFamily.GenericSansSerif; }
        }
        public static Font Make(float size,bool bold) { return new Font(family,size,bold?FontStyle.Bold:FontStyle.Regular); }
    }
}

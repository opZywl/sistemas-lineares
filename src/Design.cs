using System;
using System.Collections.Generic;
using System.Drawing;
using System.Drawing.Drawing2D;
using System.Globalization;
using System.IO;
using System.Reflection;
using System.Windows.Forms;
using System.Xml.Linq;

namespace SistemasLineares {
    internal static class Theme {
        public static readonly Color Canvas = Color.FromArgb(17,16,16);
        public static readonly Color Surface = Color.FromArgb(22,21,21);
        public static readonly Color Field = Color.FromArgb(29,28,28);
        public static readonly Color Ink = Color.FromArgb(243,240,247);
        public static readonly Color Muted = Color.FromArgb(170,165,177);
        public static readonly Color Border = Color.FromArgb(53,49,55);
        public static readonly Color Accent = Color.FromArgb(206,169,243);
        public static readonly Color Soft = Color.FromArgb(48,39,57);
        public static readonly Color Error = Color.FromArgb(255,153,145);
        public static readonly Font Small = EmbeddedFonts.Make(9F,false);
        public static readonly Font Body = EmbeddedFonts.Make(10F,false);
        public static readonly Font Medium = EmbeddedFonts.Make(10F,true);
        public static readonly Font Heading = EmbeddedFonts.Make(16F,true);
        public static readonly Font Large = EmbeddedFonts.Make(31F,true);
        public static readonly Font Value = EmbeddedFonts.Make(24F,true);
        public static readonly Font Input = EmbeddedFonts.Make(12F,false);
        public static GraphicsPath Round(RectangleF r, float radius) {
            float d = Math.Min(radius * 2, Math.Min(r.Width, r.Height));
            GraphicsPath p = new GraphicsPath();
            p.AddArc(r.X,r.Y,d,d,180,90); p.AddArc(r.Right-d,r.Y,d,d,270,90);
            p.AddArc(r.Right-d,r.Bottom-d,d,d,0,90); p.AddArc(r.X,r.Bottom-d,d,d,90,90); p.CloseFigure(); return p;
        }
        public static void Box(Graphics g, RectangleF rect, float radius, Color fill, Color stroke) {
            using (GraphicsPath p = Round(rect,radius)) {
                using(SolidBrush b = new SolidBrush(fill)) g.FillPath(b,p);
                if (stroke != Color.Empty) using(Pen pen = new Pen(stroke)) g.DrawPath(pen,p);
            }
        }
    }
    // Ícones SVG locais, incorporados ao executável. O conjunto utiliza apenas
    // primitivas vetoriais; é desenhado na escala atual, sem bitmaps intermediários.
    internal static class SvgIcons {
        private static readonly Dictionary<string,XElement> cache = new Dictionary<string,XElement>();
        private static float N(XElement e,string a,float fallback) { XAttribute v=e.Attribute(a); return v==null ? fallback : float.Parse(v.Value,CultureInfo.InvariantCulture); }
        public static void Draw(Graphics g,string name,RectangleF rect,Color color) {
            XElement svg;
            if (!cache.TryGetValue(name,out svg)) {
                using(Stream stream=Assembly.GetExecutingAssembly().GetManifestResourceStream("Icons."+name+".svg")) {
                    if(stream==null) return; svg=XElement.Load(stream); cache[name]=svg;
                }
            }
            GraphicsState state=g.Save(); g.TranslateTransform(rect.X,rect.Y); g.ScaleTransform(rect.Width/24F,rect.Height/24F);
            using(Pen pen = new Pen(color,1.65F)) {
                pen.StartCap=LineCap.Round; pen.EndCap=LineCap.Round; pen.LineJoin=LineJoin.Round;
                foreach(XElement e in svg.Elements()) {
                    string tag=e.Name.LocalName;
                    if(tag=="line") g.DrawLine(pen,N(e,"x1",0),N(e,"y1",0),N(e,"x2",0),N(e,"y2",0));
                    else if(tag=="circle") { float r=N(e,"r",1); g.DrawEllipse(pen,N(e,"cx",0)-r,N(e,"cy",0)-r,2*r,2*r); }
                    else if(tag=="rect") { float x=N(e,"x",0),y=N(e,"y",0),w=N(e,"width",1),h=N(e,"height",1),r=N(e,"rx",0); if(r>0) { using(GraphicsPath p=Theme.Round(new RectangleF(x,y,w,h),r)) g.DrawPath(pen,p); } else g.DrawRectangle(pen,x,y,w,h); }
                    else if(tag=="polyline" || tag=="polygon") {
                        string[] numbers=e.Attribute("points").Value.Split(new char[]{' ',','},StringSplitOptions.RemoveEmptyEntries); PointF[] points=new PointF[numbers.Length/2];
                        for(int i=0;i<points.Length;i++) points[i]=new PointF(float.Parse(numbers[i*2],CultureInfo.InvariantCulture),float.Parse(numbers[i*2+1],CultureInfo.InvariantCulture));
                        if(tag=="polygon") g.DrawPolygon(pen,points); else g.DrawLines(pen,points);
                    }
                }
            }
            g.Restore(state);
        }
    }
    internal sealed class IconView : Control {
        public string IconName; public Color IconColor=Theme.Accent;
        public IconView(string name) { IconName=name; SetStyle(ControlStyles.UserPaint|ControlStyles.AllPaintingInWmPaint|ControlStyles.OptimizedDoubleBuffer|ControlStyles.SupportsTransparentBackColor,true); BackColor=Color.Transparent; TabStop=false; }
        protected override void OnPaint(PaintEventArgs e) { e.Graphics.SmoothingMode=SmoothingMode.AntiAlias; SvgIcons.Draw(e.Graphics,IconName,new RectangleF(1,1,Width-2,Height-2),IconColor); }
    }
    internal sealed class Card : Panel {
        public Color Fill=Theme.Surface; public Color Stroke=Theme.Border;
        public Card() { SetStyle(ControlStyles.UserPaint|ControlStyles.AllPaintingInWmPaint|ControlStyles.OptimizedDoubleBuffer|ControlStyles.ResizeRedraw,true); BackColor=Theme.Canvas; }
        protected override void OnPaint(PaintEventArgs e) { e.Graphics.SmoothingMode=SmoothingMode.AntiAlias; Theme.Box(e.Graphics,new RectangleF(1,1,Width-3,Height-3),16,Fill,Stroke); using(GraphicsPath path=Theme.Round(new RectangleF(4,4,Width-9,Height-9),13)) using(Pen p=new Pen(Color.FromArgb(18,255,255,255))) e.Graphics.DrawPath(p,path); }
    }
    internal sealed class ActionButton : Button {
        public string IconName; public bool Primary; public bool Selected; public bool Quiet;
        private bool hover;
        public ActionButton(string text,string icon,bool primary) {
            Text=text; IconName=icon; Primary=primary; Font=Theme.Medium; Cursor=Cursors.Hand; FlatStyle=FlatStyle.Flat; FlatAppearance.BorderSize=0;
            SetStyle(ControlStyles.UserPaint|ControlStyles.AllPaintingInWmPaint|ControlStyles.OptimizedDoubleBuffer|ControlStyles.ResizeRedraw,true);
            UseVisualStyleBackColor=false; BackColor=Theme.Surface; AccessibleName=text;
        }
        protected override void OnMouseEnter(EventArgs e) { hover=true; Invalidate(); base.OnMouseEnter(e); }
        protected override void OnMouseLeave(EventArgs e) { hover=false; Invalidate(); base.OnMouseLeave(e); }
        protected override void OnGotFocus(EventArgs e) { Invalidate(); base.OnGotFocus(e); }
        protected override void OnLostFocus(EventArgs e) { Invalidate(); base.OnLostFocus(e); }
        protected override void OnPaint(PaintEventArgs e) {
            Graphics g=e.Graphics; g.Clear(Parent is Card ? ((Card)Parent).Fill : (Parent==null ? BackColor : Parent.BackColor)); g.SmoothingMode=SmoothingMode.AntiAlias;
            Color fill=Primary || Selected ? (hover ? Color.FromArgb(222,193,249) : Theme.Accent) : (hover ? Theme.Soft : BackColor);
            Color foreground= !Enabled ? Color.FromArgb(135,130,143) : (Primary || Selected ? Theme.Canvas : Theme.Ink);
            if(!Enabled) fill=Color.FromArgb(42,37,47);
            Theme.Box(g,new RectangleF(1,1,Width-3,Height-3),10,fill,Primary || Selected || Quiet ? Color.Empty : Theme.Border);
            int icon=string.IsNullOrEmpty(IconName)?0:22; Size text=TextRenderer.MeasureText(Text,Font); int total=text.Width+(icon>0?icon+8:0); int x=(Width-total)/2;
            if(icon>0) SvgIcons.Draw(g,IconName,new RectangleF(x,(Height-icon)/2,icon,icon),foreground);
            TextRenderer.DrawText(g,Text,Font,new Rectangle(x+(icon>0?icon+8:0),0,text.Width,Height),foreground,TextFormatFlags.VerticalCenter|TextFormatFlags.HorizontalCenter|TextFormatFlags.SingleLine);
            if(Focused && ShowFocusCues) { using(Pen p=new Pen(Primary?Color.FromArgb(163,216,191):Theme.Accent,1)) { p.DashStyle=DashStyle.Dot; using(GraphicsPath path=Theme.Round(new RectangleF(5,5,Width-11,Height-11),7)) g.DrawPath(p,path); } }
        }
    }
    internal sealed class NumberField : UserControl {
        public readonly TextBox Editor=new TextBox(); public string Suffix; public bool HasError;
        public NumberField(string suffix) {
            Suffix=suffix; AutoScaleMode=AutoScaleMode.None; BackColor=Theme.Surface; SetStyle(ControlStyles.AllPaintingInWmPaint|ControlStyles.OptimizedDoubleBuffer|ControlStyles.UserPaint|ControlStyles.ResizeRedraw,true);
            Editor.BorderStyle=BorderStyle.None; Editor.Font=Theme.Input; Editor.BackColor=Theme.Field; Editor.ForeColor=Theme.Ink; Editor.MaxLength=32767; Editor.TextAlign=HorizontalAlignment.Center;
            Editor.GotFocus+=delegate{Invalidate();}; Editor.LostFocus+=delegate{Invalidate();}; Controls.Add(Editor); TabStop=false; Size=new Size(110,46);
        }
        protected override void OnResize(EventArgs e) { base.OnResize(e); Editor.SetBounds(8,Math.Max(5,(Height-Editor.PreferredHeight)/2),Math.Max(15,Width-(Suffix==null?16:36)),Editor.PreferredHeight); }
        protected override void OnPaint(PaintEventArgs e) {
            Graphics g=e.Graphics; g.SmoothingMode=SmoothingMode.AntiAlias;
            Theme.Box(g,new RectangleF(1,1,Width-3,Height-3),9,Theme.Field,HasError?Theme.Error:(Editor.Focused?Theme.Accent:Theme.Border));
            if(Suffix!=null) TextRenderer.DrawText(g,Suffix,Theme.Medium,new Rectangle(Width-28,0,22,Height),Theme.Muted,TextFormatFlags.VerticalCenter|TextFormatFlags.HorizontalCenter);
        }
        protected override void OnClick(EventArgs e) { Editor.Focus(); base.OnClick(e); }
    }
}

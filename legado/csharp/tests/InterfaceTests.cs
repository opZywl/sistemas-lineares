using System;
using System.Drawing;
using System.Reflection;
using System.Windows.Forms;
using SistemasLineares;

class InterfaceTests {
    static int checks;
    static void Check(bool value,string message) { checks++; if(!value) throw new Exception(message); }
    static object Field(object o,string name) { return o.GetType().GetField(name,BindingFlags.Instance|BindingFlags.NonPublic).GetValue(o); }
    static void Call(object o,string name,params object[] args) { o.GetType().GetMethod(name,BindingFlags.Instance|BindingFlags.NonPublic).Invoke(o,args); }
    static TextBox Editor(object fields,int i,int j) { object f=((Array)fields).GetValue(i,j); return (TextBox)f.GetType().GetField("Editor").GetValue(f); }
    [STAThread] static void Main() {
        Application.EnableVisualStyles();
        using(MainForm f=new MainForm()) {
            f.CreateControl(); object fields=Field(f,"inputs");
            Check(f.Font.Name.Contains("Space Grotesk"),"fonte incorporada, sem instalação no PC");
            Check(((Control)Field(f,"canvas")).Width>=1040,"área rolável em telas pequenas");
            Check(Editor(fields,0,0).MaxLength>100,"entrada longa não é truncada no limite do cálculo");
            Check(!((Button)Field(f,"copy")).Enabled,"copy disabled initially");
            Call(f,"SolveClick",null,EventArgs.Empty);
            Check(((Label)Field(f,"helper")).Text.Contains("Equação 1"),"inline validation");
            Call(f,"FillExample"); Call(f,"SolveClick",null,EventArgs.Empty);
            string result=(string)Field(f,"lastResult"); Check(result.Contains("x = 2") && result.Contains("y = 1"),"2x2 UI result");
            Check(((Button)Field(f,"copy")).Enabled,"copy enabled after solve");
            Editor(fields,0,0).Text="4"; Check(Field(f,"lastResult")==null,"input invalidates result");
            Call(f,"ChangeDimension",3); Call(f,"FillExample"); Call(f,"SolveClick",null,EventArgs.Empty);
            result=(string)Field(f,"lastResult"); Check(result.Contains("x = 1") && result.Contains("y = 2") && result.Contains("z = 3"),"3x3 UI result");
            for(int i=0;i<3;i++) for(int j=0;j<4;j++) Editor(fields,i,j).Text="0";
            Call(f,"SolveClick",null,EventArgs.Empty); Check(((Label)Field(f,"solutionTitle")).Text=="Infinitas soluções","infinite state");
            Editor(fields,0,3).Text="1"; Call(f,"SolveClick",null,EventArgs.Empty); Check(((Label)Field(f,"solutionTitle")).Text=="Sem solução","no solution state");
            Call(f,"ClearClick",null,EventArgs.Empty); Check(Editor(fields,0,0).Text=="" && !((Button)Field(f,"copy")).Enabled,"clear resets state");
            f.Size=f.MinimumSize; Call(f,"ChangeDimension",3);
            Control lastInput=(Control)((Array)fields).GetValue(2,3); Control helper=(Control)Field(f,"helper");
            Check(lastInput.Bottom<=helper.Top,"minimum size input layout");
            CardBounds(f,"solve"); CardBounds(f,"example"); CardBounds(f,"clear"); CardBounds(f,"copy");
            Check(((Button)Field(f,"author")).AccessibleDescription.Contains("https://lucas-lima.xyz/"),"author destination");
            foreach(Control c in f.Controls) Check(c.Text!="MATEMÁTICA, COM CLAREZA","removed tagline");
            Type icons=typeof(MainForm).Assembly.GetType("SistemasLineares.SvgIcons");
            foreach(string name in new[]{"matrix","grid","copy","external","arrow-right","sparkles","reset"}) {
                using(Bitmap bmp=new Bitmap(48,48)) using(Graphics g=Graphics.FromImage(bmp)) {
                    g.Clear(Color.White); icons.GetMethod("Draw").Invoke(null,new object[]{g,name,new RectangleF(0,0,48,48),Color.Black});
                    int pixels=0; for(int x=0;x<48;x++) for(int y=0;y<48;y++) if(bmp.GetPixel(x,y).ToArgb()!=Color.White.ToArgb()) pixels++;
                    Check(pixels>20,"embedded SVG "+name);
                }
            }
        }
        Console.WriteLine("PASS: " + checks + " verificações da interface");
    }
    static void CardBounds(MainForm form,string name) { Control c=(Control)Field(form,name); Check(c.Left>=0 && c.Right<=c.Parent.Width && c.Bottom<=c.Parent.Height,"button layout: "+name); }
}

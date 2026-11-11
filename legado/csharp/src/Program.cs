using System;
using System.Diagnostics;
using System.Drawing;
using System.IO;
using System.Reflection;
using System.Runtime.InteropServices;
using System.Windows.Forms;

namespace SistemasLineares {
    public sealed class MainForm : Form {
        [DllImport("dwmapi.dll")] private static extern int DwmSetWindowAttribute(IntPtr window,int attribute,ref int value,int size);
        protected override void OnHandleCreated(EventArgs e) {
            base.OnHandleCreated(e);
            try { int dark=1; if(DwmSetWindowAttribute(Handle,20,ref dark,4)!=0) DwmSetWindowAttribute(Handle,19,ref dark,4); }
            catch(DllNotFoundException) { } catch(EntryPointNotFoundException) { }
        }
        private readonly Panel canvas=new Panel();
        private readonly NumberField[,] inputs=new NumberField[3,4];
        private readonly Label[] rowNumbers=new Label[3];
        private readonly Label[,] operators=new Label[3,3];
        private readonly Card entryCard=new Card(), solutionCard=new Card();
        private readonly Label intro, subtitle, entryTitle, entryDescription, dimensionText, gridLabel, helper;
        private readonly Label solutionLabel, solutionTitle, solutionDescription, emptyTitle, emptyDescription, resultNote;
        private readonly IconView entryIcon=new IconView("grid"), emptyIcon=new IconView("matrix");
        private readonly ActionButton two=new ActionButton("2 × 2",null,false), three=new ActionButton("3 × 3",null,false);
        private readonly ActionButton solve=new ActionButton("Resolver sistema","arrow-right",true), example=new ActionButton("Usar exemplo","sparkles",false), clear=new ActionButton("Limpar","reset",false);
        private readonly ActionButton copy=new ActionButton("Copiar resultado","copy",false), author=new ActionButton("Feito por Lucas Lima","external",false);
        private readonly Card[] valueCards=new Card[3];
        private readonly Label[] variableNames=new Label[3];
        private readonly TextBox[] variableValues=new TextBox[3];
        private readonly RichTextBox generalResult=new RichTextBox();
        private readonly ToolTip tips=new ToolTip();
        private readonly float scale;
        private int dimension=2; private bool building=true; private string lastResult; private Solution current;
        private int D(float n) { return (int)Math.Round(n*scale); }
        private void Place(Control c,int x,int y,int w,int h) { c.SetBounds(D(x),D(y),D(w),D(h)); }
        private Label TextLabel(string text,Font font,Color color,Control parent) { Label l=new Label(); l.Text=text; l.Font=font; l.ForeColor=color; l.BackColor=Color.Transparent; parent.Controls.Add(l); return l; }
        public MainForm() {
            using(Graphics g=CreateGraphics()) scale=g.DpiX/96F;
            AutoScaleMode=AutoScaleMode.None; Text="Sistemas Lineares · Lucas Lima"; BackColor=Theme.Canvas; Font=Theme.Body;
            StartPosition=FormStartPosition.CenterScreen; ClientSize=new Size(Math.Min(D(1160),Screen.PrimaryScreen.WorkingArea.Width-40),Math.Min(D(810),Screen.PrimaryScreen.WorkingArea.Height-60)); MinimumSize=new Size(640,480); DoubleBuffered=true; AutoScroll=true;
            canvas.BackColor=Theme.Canvas; Controls.Add(canvas);
            canvas.Paint+=delegate(object sender,PaintEventArgs e) { using(Pen pen=new Pen(Theme.Border)) e.Graphics.DrawLine(pen,D(32),D(86),canvas.Width-D(32),D(86)); };
            using(Stream stream=Assembly.GetExecutingAssembly().GetManifestResourceStream("App.ico")) { if(stream!=null) Icon=new Icon(stream); }
            IconView logo=new IconView("matrix"); canvas.Controls.Add(logo); Place(logo,32,29,35,35);
            Label brand=TextLabel("Sistemas Lineares",Theme.Medium,Theme.Ink,canvas); Place(brand,79,36,250,28);
            author.Quiet=true; author.BackColor=Theme.Canvas; author.AccessibleDescription="Abre https://lucas-lima.xyz/ no navegador padrão."; canvas.Controls.Add(author); author.Click+=OpenAuthor; tips.SetToolTip(author,"Visitar lucas-lima.xyz");
            intro=TextLabel("SISTEMAS\nLINEARES.",Theme.Large,Theme.Ink,canvas);
            subtitle=TextLabel("Insira os coeficientes. Confira a solução.\nCálculo exato por eliminação de Gauss-Jordan.",Theme.Body,Theme.Muted,canvas);
            canvas.Controls.Add(entryCard); canvas.Controls.Add(solutionCard); solutionCard.Fill=Theme.Accent; solutionCard.Stroke=Color.FromArgb(222,192,249);
            entryCard.Controls.Add(entryIcon);
            entryTitle=TextLabel("Seu sistema",Theme.Heading,Theme.Ink,entryCard);
            entryDescription=TextLabel("Escolha o tamanho e preencha cada equação.",Theme.Body,Theme.Muted,entryCard);
            entryCard.Controls.Add(two); entryCard.Controls.Add(three); two.Selected=true; two.Quiet=three.Quiet=true;
            two.Click+=delegate { ChangeDimension(2); }; three.Click+=delegate { ChangeDimension(3); };
            dimensionText=TextLabel("2 equações · 2 incógnitas",Theme.Small,Theme.Muted,entryCard);
            gridLabel=TextLabel("COEFICIENTES E TERMOS INDEPENDENTES",Theme.Small,Theme.Muted,entryCard);
            for(int i=0;i<3;i++) {
                rowNumbers[i]=TextLabel((i+1).ToString("00"),Theme.Small,Theme.Muted,entryCard);
                for(int j=0;j<4;j++) {
                    NumberField field=new NumberField(j==3?null:new string[]{"x","y","z"}[j]); inputs[i,j]=field;
                    field.Editor.AccessibleName="Equação "+(i+1)+", "+(j==3?"termo independente":"coeficiente de "+new string[]{"x","y","z"}[j]);
                    field.Editor.TabIndex=0; field.TabIndex=2+i*4+j; field.Editor.TextChanged+=InputChanged; entryCard.Controls.Add(field);
                }
                for(int j=0;j<3;j++) operators[i,j]=TextLabel("+",Theme.Medium,Theme.Muted,entryCard);
            }
            helper=TextLabel("Aceita 2, -1,5 e 3/4. Use 0 se a variável não aparecer.",Theme.Small,Theme.Muted,entryCard);
            entryCard.Controls.Add(solve); entryCard.Controls.Add(example); entryCard.Controls.Add(clear);
            solve.Click+=SolveClick; clear.Click+=ClearClick; example.Click+=delegate{FillExample();}; AcceptButton=solve;
            solve.TabIndex=20; example.TabIndex=21; clear.TabIndex=22; two.TabIndex=0; three.TabIndex=1; author.TabIndex=30;
            solutionLabel=TextLabel("RESULTADO",Theme.Small,Theme.Canvas,solutionCard);
            solutionTitle=TextLabel("",Theme.Heading,Theme.Canvas,solutionCard); solutionDescription=TextLabel("",Theme.Body,Color.FromArgb(58,43,70),solutionCard);
            solutionCard.Controls.Add(emptyIcon); emptyIcon.IconColor=Theme.Canvas;
            emptyTitle=TextLabel("Aguardando os valores",Theme.Heading,Theme.Canvas,solutionCard); emptyTitle.TextAlign=ContentAlignment.MiddleCenter;
            emptyDescription=TextLabel("Preencha o sistema ao lado ou use um exemplo.\nA resposta será exibida neste painel.",Theme.Body,Color.FromArgb(58,43,70),solutionCard); emptyDescription.TextAlign=ContentAlignment.TopCenter;
            for(int i=0;i<3;i++) {
                Card tile=new Card(); tile.Fill=Color.FromArgb(224,201,245); tile.BackColor=solutionCard.Fill; tile.Stroke=Color.FromArgb(186,148,219); valueCards[i]=tile; solutionCard.Controls.Add(tile);
                variableNames[i]=TextLabel(new string[]{"x","y","z"}[i],Theme.Heading,Theme.Canvas,tile);
                TextBox text=new TextBox(); text.ReadOnly=true; text.BorderStyle=BorderStyle.None; text.BackColor=tile.Fill; text.ForeColor=Theme.Canvas; text.Font=Theme.Value; text.TabStop=false; text.AccessibleName="Valor de "+new string[]{"x","y","z"}[i]; variableValues[i]=text; tile.Controls.Add(text);
            }
            generalResult.ReadOnly=true; generalResult.BorderStyle=BorderStyle.None; generalResult.BackColor=solutionCard.Fill; generalResult.ForeColor=Theme.Canvas; generalResult.Font=Theme.Input; generalResult.ScrollBars=RichTextBoxScrollBars.Vertical; generalResult.TabStop=false; generalResult.DetectUrls=false; solutionCard.Controls.Add(generalResult);
            resultNote=TextLabel("Resultados exatos, incluindo frações.",Theme.Small,Color.FromArgb(58,43,70),solutionCard);
            copy.BackColor=Theme.Surface; copy.Enabled=false; copy.Click+=CopyClick; solutionCard.Controls.Add(copy);
            Label footer=TextLabel("Gauss-Jordan  /  aritmética racional exata",Theme.Small,Theme.Muted,canvas); footer.Name="footer";
            Label shortcut=TextLabel("ENTER  para resolver     ·     TAB  para navegar",Theme.Small,Theme.Muted,canvas); shortcut.Name="shortcut"; shortcut.TextAlign=ContentAlignment.TopRight;
            building=false; LayoutScreen(); ResetResult(); Shown+=delegate{inputs[0,0].Editor.Focus();};
        }
        protected override void OnResize(EventArgs e) { base.OnResize(e); if(!building && intro!=null) LayoutScreen(); }
        private void LayoutScreen() {
            int width=Math.Max(1040,(int)((ClientSize.Width-SystemInformation.VerticalScrollBarWidth)/scale)), height=Math.Max(760,(int)((ClientSize.Height-SystemInformation.HorizontalScrollBarHeight)/scale));
            canvas.Size=new Size(D(width),D(height));
            Place(author,width-264,28,232,42); Place(intro,30,104,370,115); Place(subtitle,418,156,width-450,62);
            int left=(int)((width-84)*0.59F), right=width-84-left, cardHeight=height-300;
            Place(entryCard,32,238,left,cardHeight); Place(solutionCard,52+left,238,right,cardHeight);
            Place(entryIcon,26,26,26,26); Place(entryTitle,65,20,left-90,33); Place(entryDescription,26,59,left-52,25);
            Place(two,26,103,88,40); Place(three,120,103,88,40); Place(dimensionText,228,113,left-250,24); Place(gridLabel,26,168,left-52,22);
            int fieldWidth=(left-26-26-29-dimension*26)/(dimension+1);
            for(int i=0;i<3;i++) {
                int y=194+i*50; Place(rowNumbers[i],26,y+14,25,25); rowNumbers[i].Visible=i<dimension; int x=55;
                for(int j=0;j<dimension;j++) { Place(inputs[i,j],x,y,fieldWidth,46); x+=fieldWidth; Place(operators[i,j],x+2,y+9,22,28); operators[i,j].Text=j==dimension-1?"=":"+"; x+=26; }
                Place(inputs[i,3],x,y,fieldWidth,46);
                for(int j=0;j<4;j++) inputs[i,j].Visible=i<dimension && (j<dimension || j==3);
                for(int j=0;j<3;j++) operators[i,j].Visible=i<dimension && j<dimension;
            }
            Place(helper,26,cardHeight-113,left-52,42); Place(solve,26,cardHeight-64,192,42); Place(example,228,cardHeight-64,155,42); Place(clear,left-115,cardHeight-64,89,42);
            Place(solutionLabel,26,26,right-52,24); Place(solutionTitle,26,65,right-52,35); Place(solutionDescription,26,106,right-52,52);
            Place(emptyIcon,(right-76)/2,125,76,76); Place(emptyTitle,20,227,right-40,40); Place(emptyDescription,25,277,right-50,64);
            for(int i=0;i<3;i++) { Place(valueCards[i],26,168+i*62,right-52,58); Place(variableNames[i],18,9,35,40); Place(variableValues[i],68,7,right-140,46); }
            Place(generalResult,26,170,right-52,cardHeight-286); Place(resultNote,26,cardHeight-110,right-52,38); Place(copy,26,cardHeight-64,right-52,42);
            Place(canvas.Controls["footer"],33,height-43,470,25); Place(canvas.Controls["shortcut"],width-485,height-43,450,25); canvas.Invalidate();
        }
        private void ChangeDimension(int n) { dimension=n; two.Selected=n==2; three.Selected=n==3; two.Invalidate(); three.Invalidate(); dimensionText.Text=n+" equações · "+n+" incógnitas"; ResetErrors(); LayoutScreen(); ResetResult(); }
        private void ResetErrors() { helper.Text="Aceita 2, -1,5 e 3/4. Use 0 se a variável não aparecer."; helper.ForeColor=Theme.Muted; for(int i=0;i<3;i++) for(int j=0;j<4;j++) { inputs[i,j].HasError=false; inputs[i,j].Invalidate(); } }
        private void InputChanged(object sender,EventArgs e) { if(building) return; ResetErrors(); ResetResult(); }
        private void ResetResult() { current=null; lastResult=null; solutionTitle.Visible=solutionDescription.Visible=false; emptyIcon.Visible=emptyTitle.Visible=emptyDescription.Visible=true; for(int i=0;i<3;i++) valueCards[i].Visible=false; generalResult.Visible=false; resultNote.Text="Resultados exatos, incluindo frações."; copy.Enabled=false; copy.Text="Copiar resultado"; }
        private void SolveClick(object sender,EventArgs e) {
            ResetErrors(); Rational[,] values=new Rational[dimension,dimension+1];
            for(int i=0;i<dimension;i++) for(int j=0;j<=dimension;j++) {
                NumberField field=inputs[i,j==dimension?3:j];
                try { values[i,j]=Rational.Parse(field.Editor.Text); }
                catch(FormatException ex) { helper.Text="Equação "+(i+1)+", "+(j==dimension?"termo independente":"coeficiente de "+new string[]{"x","y","z"}[j])+": "+ex.Message; helper.ForeColor=Theme.Error; field.HasError=true; field.Invalidate(); field.Editor.Focus(); field.Editor.SelectAll(); return; }
            }
            current=Solver.Solve(values); lastResult=Solver.Format(current); copy.Enabled=true; copy.Text="Copiar resultado";
            emptyIcon.Visible=emptyTitle.Visible=emptyDescription.Visible=false; solutionTitle.Visible=solutionDescription.Visible=true;
            for(int i=0;i<3;i++) valueCards[i].Visible=false; generalResult.Visible=current.Kind!=SolutionKind.Unique;
            if(current.Kind==SolutionKind.Unique) {
                solutionTitle.Text="Solução encontrada"; solutionDescription.Text="Estes valores satisfazem todas\nas equações do sistema.";
                for(int i=0;i<dimension;i++) { valueCards[i].Visible=true; variableValues[i].Text=current.Reduced[i,dimension].ToString(); variableValues[i].SelectionStart=0; tips.SetToolTip(variableValues[i],variableValues[i].Text); }
                resultNote.Text="Solução única · valores exatos";
            } else if(current.Kind==SolutionKind.Infinite) {
                solutionTitle.Text="Infinitas soluções"; solutionDescription.Text="As equações são dependentes.\nVeja a forma geral da resposta:";
                string formatted=lastResult; int start=formatted.IndexOf("\r\n\r\n",StringComparison.Ordinal); generalResult.Text=start>=0?formatted.Substring(start+4):formatted; resultNote.Text="Os parâmetros t1, t2, … são números reais livres.";
            } else { solutionTitle.Text="Sem solução"; solutionDescription.Text="As equações são incompatíveis."; generalResult.Text="Não existe um conjunto de valores que satisfaça todas as equações ao mesmo tempo."; resultNote.Text="Confira os coeficientes se esse não era o esperado."; }
        }
        private void FillExample() { string[,] s2={{"2","1","5"},{"1","-1","1"}}; string[,] s3={{"1","1","1","6"},{"2","-1","1","3"},{"1","2","-1","2"}}; for(int i=0;i<dimension;i++) for(int j=0;j<=dimension;j++) inputs[i,j==dimension?3:j].Editor.Text=dimension==2?s2[i,j]:s3[i,j]; ResetErrors(); ResetResult(); helper.Text="Exemplo pronto. Clique em Resolver sistema ou pressione Enter."; solve.Focus(); }
        private void ClearClick(object sender,EventArgs e) { for(int i=0;i<3;i++) for(int j=0;j<4;j++) inputs[i,j].Editor.Clear(); ResetErrors(); ResetResult(); inputs[0,0].Editor.Focus(); }
        private void CopyClick(object sender,EventArgs e) { if(lastResult==null) return; try { Clipboard.SetText(lastResult); copy.Text="Resultado copiado"; } catch(System.Runtime.InteropServices.ExternalException) { resultNote.Text="Não foi possível copiar agora. Tente novamente."; } }
        private void OpenAuthor(object sender,EventArgs e) { try { Process.Start(new ProcessStartInfo("https://lucas-lima.xyz/") { UseShellExecute=true }); } catch(System.ComponentModel.Win32Exception) { helper.Text="Abra lucas-lima.xyz no seu navegador para visitar o autor."; } }
        protected override void Dispose(bool disposing) { if(disposing) tips.Dispose(); base.Dispose(disposing); }
        [STAThread] public static void Main() { Application.EnableVisualStyles(); Application.SetCompatibleTextRenderingDefault(false); Application.Run(new MainForm()); }
    }
}

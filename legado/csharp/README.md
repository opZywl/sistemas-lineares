# Versão C# (legado)

Esta é a versão 3 do Sistemas Lineares, em C# com WinForms e .NET Framework 4. Ela foi substituída pelo app nativo em Rust (`native/`) e fica aqui como implementação de referência independente.

O `testar-tudo.cmd` da raiz compila a auditoria desta versão e confere que ela e o núcleo Rust dão as mesmas 76.674 respostas, byte a byte.

Para compilar e testar só esta versão, rode `compilar.cmd`. O resultado é `legado\csharp\SistemasLineares.exe`, que não vai para o git.

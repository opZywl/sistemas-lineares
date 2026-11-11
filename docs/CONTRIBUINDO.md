# Contribuindo com o Sistemas Lineares

Obrigado pelo interesse em contribuir. Este guia explica como preparar o ambiente, as regras do código e o que todo pull request precisa passar.

## Preparando o ambiente
Siga o [AMBIENTE.md](AMBIENTE.md). Para mudanças grandes, abra antes uma [issue](https://github.com/opZywl/sistemas-lineares/issues) para combinar a ideia.

## Regras do código
- **Sem comentários no código**, nem doc comments longos. Nomes claros e funções pequenas explicam o código.
- **Textos para o usuário em PT-BR**, com acentos corretos e os mesmos caracteres das versões atuais, como `·`, `×` e `…`.
- **As duas versões andam juntas:** o app nativo e o HTML têm os mesmos textos, as mesmas regras de entrada e as mesmas respostas.
- **Sem `unwrap` ou `expect`** onde o app pode falhar em execução. Trate o erro e mostre algo útil, nunca uma janela em branco.
- **API estável no núcleo:** o app depende de `native/core`, então mudanças na API pública precisam de motivo forte.
- **Dependências novas** só com justificativa no pull request.
- **Formatação e Clippy:** rode `cargo fmt --all` e deixe o `cargo clippy --all-targets --locked -- -D warnings` sem avisos, ambos dentro de `native/`.

## Testes e paridade (obrigatórios)
Todo pull request precisa terminar o `testar-tudo.cmd` com **OK**. O workflow Build roda o mesmo script, e o Lint confere a formatação, o Clippy e o HTML gerado.

- **Mudou a matemática, a leitura dos números ou o texto da resposta?** Faça a mesma mudança em `native/core`, em `portable/solver.js` e em `legado/csharp/src/Solver.cs`. A paridade compara as 76.674 respostas byte a byte entre as três versões, então mudar só uma delas quebra a validação.
- **Mudou `portable/` ou `assets/`?** Rode `node tools/build-portable.cjs` e faça commit do `SistemasLineares.html` gerado.
- **Comportamento novo pede teste novo:** o núcleo é testado em `native/core/tests`; o estado e o layout do app, em `native/app`.
- **Mudança visual:** anexe ao pull request capturas de antes e depois.

## Mensagens de commit
O projeto usa [Conventional Commits](https://www.conventionalcommits.org/pt-br/v1.0.0/) em PT-BR, com o escopo entre parênteses depois dos dois-pontos:

```
<tipo>: (<escopo>) <descrição>
```

Os tipos são `feat`, `fix`, `docs`, `style`, `refactor`, `perf`, `test`, `build`, `ci` e `chore`. Os escopos mais comuns são `core`, `app`, `html`, `legado`, `docs`, `ci` e `repo`.

```
feat: (app) lembra a dimensão 2×2 ou 3×3 entre aberturas
fix: (core) rejeita expoente fora de -1000 a 1000 com a mensagem certa
docs: (readme) explica como conferir o SHA-256
```

- Escreva a descrição curta, no presente e sem ponto final.
- Adicione os arquivos pelo caminho (`git add caminho/do/arquivo`), nunca com `git add -A`, para que binários e pastas de build não entrem no git.
- Não inclua `Co-Authored-By` de ferramentas automáticas.

## Pull requests
1. Crie uma branch a partir da `main`: `git checkout -b feat/minha-mudanca`.
2. Faça a mudança seguindo as regras acima.
3. Rode o `testar-tudo.cmd`, o `cargo fmt --all` e o Clippy.
4. Registre a mudança na seção **[Não lançado]** do [CHANGELOG.md](../CHANGELOG.md).
5. Abra o pull request para a `main` dizendo o que mudou, por que mudou e como foi testado.

## Publicando uma versão
1. Atualize a versão em `native/Cargo.toml`, na seção `[workspace.package]`; em `native/app/app.rc`, em `FILEVERSION`, `PRODUCTVERSION` e nos textos `FileVersion` e `ProductVersion`; e no `assemblyIdentity` de `native/app/app.manifest`.
2. No CHANGELOG, troque **[Não lançado]** por `[X.Y.Z] - AAAA-MM-DD`, abra uma nova seção **[Não lançado]** vazia acima dela e atualize os links do fim do arquivo.
3. Faça o commit, crie a tag e envie: `git tag vX.Y.Z` e `git push origin vX.Y.Z`.
4. O workflow Release confere se a tag bate com o `Cargo.toml`, o `app.rc`, o `app.manifest` e o CHANGELOG, roda a validação completa, compila e publica a release com o exe, o HTML, o `SHA256SUMS.txt` e as notas tiradas do CHANGELOG.

## Dúvidas
Abra uma [issue](https://github.com/opZywl/sistemas-lineares/issues) com a sua pergunta ou veja a [arquitetura](ARQUITETURA.md) do projeto.

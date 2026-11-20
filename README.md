# ✂ resenha

**Subsistema de Extração, Sanitização e Destilação Tipográfica na Borda**

* **Ecossistema:** 2LP // A Banca (`fuxico.lumen.pink` / `resenha`)
* **Categoria:** 🌐 IndieWeb / ✂ Editorial
* **Linguagem:** Rust (Edição 2021)
* **Licença:** [GPL-3.0](./LICENSE)
* **Governança:** Norma [GOST](.gost/GOST.md) v3.0.0 (Perfil Kollontai 2.1.0 / Lilya 2.1.0)

---

## 1. Manifesto

O **`resenha`** é o braço editorial da Banca: um microserviço e utilitário CLI em Rust puro, ultrarrápido (< 15 MB RAM). Ele extrai o miolo textual de páginas web abertas e entrega em **Markdown estrito** com frontmatter e preservação da fonte original.

Opera como CLI independente ou motor integrado ao **Fuxico** (`fuxico.lumen.pink`).

---

## 2. Instalação & Uso

```bash
# Extrair direto no terminal ou salvar Markdown
resenha "https://exemplo.com/noticia" -o artigo.md

# Iniciar servidor localmente ou via container
resenha serve --port 3030
docker compose up -d
```

Detalhes completos de uso e variáveis em [docs/usage.md](./docs/usage.md).

---

## 3. Governança GOST

Gerenciado estritamente pelas diretrizes **GOST DD-TDD** e pela ferramenta **CCCP**:

```bash
cccp check --strict
```

## 📄 Licença & Matriz Modular

Em conformidade com a norma técnica [GOST v3.0.0](.gost/GOST.md):

* **Código-Fonte e Binário CLI (`src/`):** Licenciados sob a [GPL-3.0](./LICENSE).
* **Identidade Visual e Emblema:** Licenciados sob a [Creative Commons Atribuição-SemDerivações 4.0 Internacional (CC-BY-ND 4.0)](LICENSE-LOGO).

---

Feito com amor travesti por [Lumen Pink](https://lumen.pink).

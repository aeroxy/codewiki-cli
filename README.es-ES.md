

# codewiki

[![crates.io](https://img.shields.io/crates/v/codewiki-cli.svg)](https://crates.io/crates/codewiki-cli)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](./LICENSE)

Consulta los wikis de repositorios de GitHub a través de [Google Code Wiki](https://codewiki.google/) sin abrir un navegador.

Diseñado para agentes de programación basados en LLM y para humanos. Imprime Markdown en la salida estándar (stdout), con referencias al código fuente de GitHub resueltas como URLs clicables y diagramas de arquitectura preservados como bloques de código delimitados ` ```dot `.

## Instalación

### Homebrew (macOS arm64)

```bash
brew install aeroxy/tap/codewiki-cli
```

### Cargo

```bash
cargo install codewiki-cli
```

El crate es `codewiki-cli`; el binario que instala es `codewiki`.

## Uso

```bash
codewiki structure facebook/react           # list section titles
codewiki read facebook/react                # full wiki as Markdown
codewiki ask facebook/react "How does useEffect work?"
```

Redirige la salida a tu agente preferido:

```bash
codewiki read ast-grep/ast-grep | claude -p "Summarise the rule engine"
```

## Cómo funciona

Code Wiki no tiene una API pública. `codewiki` se comunica con el RPC `batchexecute` de Google de la misma manera que lo hace la interfaz web:

- `VSX6ub` devuelve todo el wiki de un repositorio como JSON estructurado (la página se renderiza del lado del servidor a partir de esta misma llamada).
- `EgIxfe` responde a preguntas de chat con Gemini.

Una caché en disco de 6 horas para la etiqueta de compilación / ID de sesión (`~/Library/Caches/codewiki/bootstrap.json` en macOS, equivalente en Linux/Windows) permite que las invocaciones consecutivas omitan la solicitud GET inicial. Puedes anular la ubicación de la caché con `$CODEWIKI_CACHE_DIR`.

No se requiere autenticación. Solo repositorios públicos de GitHub (Code Wiki aún no soporta repositorios privados).

### TLS

La verificación de certificados TLS está **desactivada de forma predeterminada**. `codewiki` está diseñado para ejecutarse dentro de entornos aislados (sandboxes) de agentes monitoreados, cuyos proxies de interceptación TLS presentan certificados que no se encadenan a una autoridad de certificación de confianza; de lo contrario, la verificación estricta haría fallar cada solicitud con un error opaco. Establece `CODEWIKI_TLS_VERIFY=1` (o `true`/`yes`) para restaurar la verificación estricta de certificados.

## Formato de salida

Cada comando imprime una línea de encabezado seguida del resultado:

```
## CodeWiki: <owner>/<repo> (<command>)

<content>
```

`read` reescribe las referencias `[`text`](%2Fowner%2Frepo%2Fpath)` a URLs absolutas `https://github.com/owner/repo/path` y emite cualquier diagrama de Graphviz incrustado después de su sección como bloques de código delimitados `dot`.

## Habilidad para Claude Code

Se incluye una habilidad para Claude Code lista para instalar en `skill/codewiki/` para que Claude sepa cuándo recurrir a `codewiki` automáticamente. Instálala:

```bash
cp -r skill/codewiki ~/.claude/skills/
```

(o crea un enlace simbólico de `skill/codewiki` a `~/.claude/skills/codewiki` si deseas seguir los cambios del proyecto original.)

## Pruebas

```bash
cargo test
```

Las pruebas de integración están controladas por `CODEWIKI_MOCK_TEXT`, por lo que el conjunto de pruebas se ejecuta sin conexión.

## Licencia

MIT

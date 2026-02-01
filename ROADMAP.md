# NovaType Roadmap

## Version actuelle : v0.1.0 ✅

### Fonctionnalités livrées
- **CLI** : `nova init`, `compile`, `watch`, `validate`, `template`
- **Compilation native** : Typst intégré comme bibliothèque
- **WASM** : Compilation navigateur pour démos interactives
- **Templates** : article, ieee-article, nature-article, report, book, cv, letter, presentation
- **Citations** : BibTeX + CrossRef API (6 styles)
- **Graphiques** : nova-plot (line, bar, scatter, pie, area) depuis CSV/JSON
- **Validation** : Schémas JSON pour métadonnées frontmatter
- **Distribution** : crates.io, npm, binaires (Linux, Windows, macOS)

---

## v0.2.0 - Intégration Python & Jupyter

### Matplotlib intégré
- [ ] Nouveau crate `nova-python`
- [ ] Exécution de blocs Python dans les documents
- [ ] Capture automatique des figures matplotlib en SVG
- [ ] Cache intelligent (hash du code source)
- [ ] Support des dépendances (numpy, pandas, scipy, etc.)

```typst
#python-figure(```python
import matplotlib.pyplot as plt
import numpy as np
x = np.linspace(0, 2*np.pi, 100)
plt.plot(x, np.sin(x))
```)
```

### Intégration Jupyter Notebook
- [ ] Import de fichiers `.ipynb`
- [ ] Conversion automatique en document NovaType
- [ ] Option pour masquer les cellules de code
- [ ] Conservation des outputs (figures, tableaux, résultats)
- [ ] Support des cellules Markdown

```bash
nova compile notebook.ipynb --hide-code --output rapport.pdf
```

---

## v0.3.0 - Expérience développeur

### Extension VSCode
- [ ] Preview live côte-à-côte
- [ ] Coloration syntaxique améliorée
- [ ] Snippets pour templates courants
- [ ] Intégration avec le terminal (watch mode)

### LSP (Language Server Protocol)
- [ ] Autocomplétion
- [ ] Erreurs en temps réel
- [ ] Go to definition
- [ ] Hover documentation
- [ ] Formatting

### Amélioration des erreurs
- [ ] Messages d'erreur contextuels
- [ ] Suggestions de correction
- [ ] Localisation (français, anglais)

---

## v0.4.0 - Export & Interopérabilité

### Formats d'export
- [ ] HTML standalone
- [ ] EPUB (ebooks)
- [ ] DOCX (Word)
- [ ] Markdown

### Intégrations
- [ ] Zotero (import bibliographies)
- [ ] Mendeley
- [ ] Import LaTeX basique

---

## v0.5.0 - Écosystème

### Système de packages
- [ ] Registry de templates communautaires
- [ ] Packages de fonctions réutilisables
- [ ] Gestion des versions

### Plus de templates
- [ ] Thèse de doctorat
- [ ] Mémoire de master
- [ ] Beamer-style slides
- [ ] Poster scientifique
- [ ] Lettre de motivation

---

## Vision long terme

### Éditeur web
- [ ] Interface type Overleaf
- [ ] Collaboration temps réel
- [ ] Historique des versions
- [ ] Commentaires et révisions

### API Cloud
- [ ] Compilation as a Service
- [ ] Webhooks pour CI/CD
- [ ] Intégration GitHub/GitLab

---

## Contribuer

Les contributions sont les bienvenues ! Voir [CONTRIBUTING.md](CONTRIBUTING.md) pour les guidelines.

Priorités actuelles :
1. 🔥 Intégration Python/Matplotlib
2. 🔥 Support Jupyter Notebook
3. Extension VSCode

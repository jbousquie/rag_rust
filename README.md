# Proxy RAG Rust

Ce projet implémente un **proxy RAG (Retrieval-Augmented Generation)** simple en **Rust**. Son objectif est de servir d'intermédiaire entre un client (comme un CLI ou un agent IA comme Zed) et un LLM distant (dans votre cas, une instance Qwen3-Coder derrière un reverse-proxy OpenAI API). Le proxy récupère des informations pertinentes à partir d'une base de connaissances vectorielle locale avant d'envoyer la requête enrichie au LLM distant, améliorant ainsi la pertinence des réponses.

> **Note :** Ce projet est conçu comme un **Proof of Concept (PoC)** pour tester l'approche RAG en Rust, en remplacement d'une implémentation Python existante.

## Fonctionnalités

*   **Proxy RAG Local :** Intercepte les requêtes du client, effectue une recherche RAG, puis transmet la requête enrichie au LLM distant.
*   **Indexation Locale :** Lit et indexe des documents (formats texte, PDF, DOCX, etc.) dans une base de connaissances vectorielle locale.
*   **Génération d'Embeddings Locaux :** Utilise une instance [Ollama](https://ollama.ai/) locale (modèle `Qwen3-Embeddings`) pour générer les embeddings nécessaires à l'indexation et à la recherche.
*   **Recherche Vectorielle :** Effectue une recherche sémantique dans la base de connaissances vectorielle locale.
*   **Appel LLM Distant :** Transmet la question d'origine enrichie du contexte récupéré à un LLM distant via une API compatible OpenAI.
*   **Séparation des Responsabilités :** Le code est organisé en deux composants principaux : un outil d'indexation et un serveur proxy.

## Changements Réalisés pour la Compilation

Les modifications suivantes ont été apportées pour permettre la compilation du projet :

1. **Correction des fonctions main asynchrones** : Les fonctions `main` dans `src/indexing/main.rs` et `src/rag_proxy/main.rs` ont été converties de `async` à `sync` pour éviter les erreurs de compilation liées à l'utilisation incorrecte de `async` dans les binaires.
2. **Correction des imports de modules** : Les imports dans `src/indexing/main.rs` ont été mis à jour pour utiliser le bon chemin de module (`rag_rust::common::Config` au lieu de `crate::common::Config`).
3. **Création du module commun** : Le module `src/common/mod.rs` a été créé pour centraliser la structure `Config` et les types partagés.
4. **Suppression du fichier main.rs redondant** : Le fichier `src/main.rs` a été supprimé car il causait des conflits de module avec les binaires.

## Prérequis
# Fichier de Configuration

Le projet utilise un fichier central de configuration `config.toml` qui permet de définir toutes les options de configuration du proxy RAG. Ce fichier contient les paramètres suivants :

- Configuration des sources de données (chemin vers le dossier des documents)
- Paramètres du proxy RAG (port et host de l'écoute)
- Configuration de l'API LLM (endpoint, modèle, clé d'API)
- Configuration de Qdrant (host, port, clé d'API)

Le fichier de configuration permet de centraliser la configuration de l'application et d'éviter la configuration manuelle via les variables d'environnement ou les arguments de ligne de commande.

*   **Rust :** [Installez Rust](https://www.rust-lang.org/tools/install) (version 1.70.0 ou supérieure recommandée).
*   **Ollama :** Doit être installé et en cours d'exécution sur la machine locale. Le modèle `Qwen3-Embeddings` doit être disponible :
    ```bash
    curl -fsSL https://ollama.ai/install.sh | sh
    ollama pull Qwen3-Embeddings
    ```
*   **Base Vectorielle :** (Pour l'instant, Qdrant est prévu) [Installez Qdrant localement](https://github.com/qdrant/qdrant?tab=readme-ov-file#quick-start) (soit via binaire, soit via Docker).
    *   *Exemple avec binaire :* Téléchargez la dernière release depuis [https://github.com/qdrant/qdrant/releases](https://github.com/qdrant/qdrant/releases), extrayez et exécutez `./qdrant`.
    *   *Exemple avec Docker :*
        ```bash
        docker run -p 6333:6333 -p 6334:6334 qdrant/qdrant
        ```
*   **LLM Distant :** Accès à une instance Qwen3-Coder (ou similaire) via une API compatible OpenAI, accessible via votre reverse-proxy.

## Stack Technique

*   **Langage :** [Rust](https://www.rust-lang.org/)
*   **Serveur HTTP :** [axum](https://crates.io/crates/axum)
*   **Lecture de fichiers :** `tokio::fs`, [mupdf-rs](https://crates.io/crates/mupdf-rs) (PDF), [docx-rs](https://crates.io/crates/docx-rs) (DOCX)
*   **Découpage de texte (Chunking) :** [text-splitter](https://crates.io/crates/text-splitter) (ou logique manuelle)
*   **Appels HTTP (Ollama, LLM distant) :** [reqwest](https://crates.io/crates/reqwest)
*   **Base de Données Vectorielle :** [qdrant-client](https://crates.io/crates/qdrant-client)
*   **(Optionnel) Appel LLM distant (OpenAI API) :** [openai-rs](https://crates.io/crates/openai-rs) (si compatible avec votre reverse-proxy)

## Structure du Projet

```text
.
├── Cargo.toml          # Dépendances et définition des binaires
├── src/
│   ├── lib.rs          # Fonctions utilitaires partagées
│   ├── indexing/       # Logique d'indexation
│   │   ├── mod.rs
│   │   ├── loader.rs   # Chargement des fichiers
│   │   ├── chunker.rs  # Découpage du texte
│   │   └── indexer.rs  # Génération embeddings (Ollama) + Stockage (Qdrant)
│   ├── rag_proxy/      # Logique du serveur proxy RAG
│   │   ├── mod.rs
│   │   ├── server.rs   # Démarrage du serveur axum
│   │   ├── handler.rs  # Gestion d'une requête : Recherche RAG -> Appel LLM -> Réponse
│   │   ├── retriever.rs # Recherche dans Qdrant
│   │   └── llm_caller.rs # Appel au LLM distant
│   └── common/         # Types, constantes partagées
│       └── mod.rs
├── data_sources/       # Dossier source pour les documents à indexer
└── ...

## Installation et Démarrage

Assurez-vous que les prérequis (Rust, Ollama, Qdrant) sont installés et en cours d'exécution.
Clonez ce dépôt :
```shell
git clone <URL_DE_VOTRE_DEPOT>
cd votre_proxy_rag
```

Compilez les binaires :
```shell
cargo build --release
```
Indexez vos documents : Placez vos documents dans le dossier data_sources/. Puis, exécutez le binaire d'indexation (à implémenter, par exemple via un argument ou un sous-binaire séparé). Cela enverra les embeddings à Qdrant.
```shell
cargo run --bin index_documents
# OU
# ./target/release/index_documents (si compilé en --release)
```
Lancez le serveur proxy : Configurez les variables d'environnement nécessaires (clé API du LLM distant, URL du LLM distant, URL de Qdrant, etc.) dans un fichier .env ou directement dans votre environnement. Ensuite, exécutez le binaire du proxy.
```shell
cargo run --bin rag_proxy
# OU
# ./target/release/proxy_server (si compilé en --release)
```
Configurez votre client (CLI, Zed, etc.) pour qu'il envoie ses requêtes au serveur proxy démarré (par exemple, http://localhost:3000 si axum écoute sur ce port).

## Étapes Suivantes / Extensibilité

* Re-ranking : Grâce à l'utilisation de Qdrant, l'intégration future de fonctionnalités de re-ranking natives est possible.
* Support d'autres formats de documents : Ajouter des crates pour lire d'autres formats (PowerPoint, etc.).
* Sécurité : Ajouter de l'authentification/autorisation si nécessaire.
* Monitoring : Intégrer des outils de logging et de métriques.

## Licence

Ce projet est sous licence MIT.

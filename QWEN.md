# Role
As a senior Rust developer, my core task is to analyze user edits and rewrite provided code excerpts, incorporating suitable suggestions based on cursor location. I prioritize writing efficient, readable, and maintainable Rust code, always adhering to best practices and ensuring thorough documentation.

I am responsible for testing and debugging to deliver error-free code that meets project requirements. When codebases grow, I propose refactoring into smaller, manageable functions and even splitting code into multiple files for better organization. Each file would contain functions related to a specific project aspect. Each time I add or modify a function, I add initial comments explaining the purpose and usage of the function. Each time I add a feature or modify an existing one or each time I refactor code, I ensure that the code remains well-organized and easy to understand and I update the file QWEN.md and possibly README.md.

I meticulously manage imports and dependencies, ensuring they are well-organized and updated during refactoring. If new dependencies are needed, I propose adding them to Cargo.toml and verify compatibility. My goal is to centralize imports and dependencies whenever possible to enhance readability and maintainability. I never hardcode values but rather use constants from a configuration file. I add comments in every module and above each function to explain its purpose and usage.

I don't implement the project all at once, but rather in small, manageable steps under the guidance of the developer. I propose the developer a plan of steps to follow. I wait for the developer's instructions before proceeding on each step.

I don't run the code to test it, I just build it. The developer will run the code to test it.

I use the agentic tools like edit_file or patch to modify the code. If needed, I can also run commands from the shell, like cd, cat, printf, sed.

# Description Technique du Projet RAG-Proxy en Rust

## 1. Objectif du Projet

Ce projet vise à créer un service de RAG (Retrieval-Augmented Generation) performant et sécurisé, entièrement développé en Rust. Le service a deux fonctions principales :

1.  **Indexer des documents** : Analyser une collection de documents (fichiers texte, PDF, DOCX, etc.), les découper, générer des représentations vectorielles (embeddings) de chaque fragment, et les stocker dans une base de données vectorielle (Qdrant).
2.  **Exposer un proxy RAG** : Mettre en place un serveur HTTP qui accepte des requêtes (questions), recherche les informations les plus pertinentes dans la base de données vectorielle, et utilise ces informations comme contexte pour interroger un grand modèle de langage (LLM) distant afin de générer une réponse précise et contextuellement informée.

## 2. Architecture Générale

Le système est divisé en deux processus distincts :

-   **Le Binaire d'Indexation (`index_documents`)** : Un outil en ligne de commande qui parcourt le répertoire `data_sources/`, traite les fichiers qu'il contient et peuple la base de données Qdrant.
-   **Le Serveur Proxy RAG (`rag_proxy`)** : Un service HTTP long-running qui expose un point d'accès (endpoint) pour les requêtes des utilisateurs.

Le flux d'une requête utilisateur est le suivant :
1.  Un client envoie une requête HTTP (contenant une question) au serveur `rag_proxy`.
2.  Le serveur transforme la question en un vecteur d'embedding.
3.  Il utilise ce vecteur pour interroger la base de données Qdrant et récupérer les fragments de documents les plus similaires (le contexte).
4.  Il construit une invite (prompt) enrichie, combinant la question originale de l'utilisateur et le contexte récupéré.
5.  Il envoie cette invite au service LLM distant.
6.  La réponse du LLM est renvoyée au client.

## 3. Organisation du Code

-   **Configuration Centrale** : Toute la configuration du projet est centralisée dans le fichier `config.toml`. Ce fichier contient tous les paramètres nécessaires : chemins des sources de données, paramètres du proxy RAG (port et host), configuration de l'API LLM (endpoint, modèle, clé d'API), et configuration de Qdrant (host, port, clé d'API).

Le projet est structuré en modules clairs pour séparer les responsabilités :

-   `Cargo.toml` : Définit les métadonnées du projet et ses dépendances (axum, reqwest, qdrant-client, tokio, tracing, etc.).
-   `src/lib.rs` : Contient du code partagé et des utilitaires qui peuvent être utilisés par les deux binaires (configuration, gestion des erreurs avec `AppError`, initialisation du logging). Définit l'enum `AppError` qui centralise tous les types d'erreurs possibles et implémente `IntoResponse` pour une intégration fluide avec Axum.

### `src/clients/`
Module contenant les clients API centralisés pour éviter la duplication de code :
-   `ollama.rs` : Client pour Ollama (génération d'embeddings)
-   `llm.rs` : Client pour le LLM distant (chat completions)

### `src/indexing/`
Ce module gère tout le processus de transformation des documents bruts en vecteurs stockés.
-   `loader.rs` : Chargement de différents types de fichiers (texte, PDF, DOCX) depuis le répertoire `data_sources/`. Utilise une architecture trait-based avec `DocumentLoader` implémenté par `TextLoader`, `PdfLoader`, et `DocxLoader` pour une extensibilité facile.
-   `chunker.rs` : Découpage des textes en fragments de taille gérable.
-   `indexer.rs` : Génération des embeddings via `OllamaClient` et stockage dans Qdrant.
-   `file_tracker.rs` : Suivi des fichiers indexés pour éviter le retraitement des fichiers non modifiés.
-   `main.rs` : Point d'entrée du binaire d'indexation.

### `src/rag_proxy/`
Ce module contient toute la logique du serveur HTTP.
-   `server.rs` : Configure et lance le serveur web `axum`, définit les routes et attache les gestionnaires (handlers). La configuration est chargée une fois et partagée via `State<Arc<Config>>`.
-   `handler.rs` : Logique principale de traitement d'une requête HTTP. Utilise `LlmClient` pour communiquer avec le LLM distant.
-   `retriever.rs` : Gère l'interaction avec Qdrant. Utilise `OllamaClient` pour générer les embeddings de la question.
-   `passthrough_handler.rs` : Gère les requêtes en mode 'passthrough' (sans traitement RAG) pour le débogage.
-   `main.rs` : Point d'entrée du binaire du proxy RAG.

### `src/qdrant_custom_client.rs`
Client personnalisé pour interagir avec Qdrant. Fournit des fonctionnalités pour tester la connectivité, vérifier l'existence de collections, créer des collections et insérer des points (vecteurs).

### `src/reset_documents/`
Binaire permettant de réinitialiser complètement la base de connaissances vectorielle (suppression de la collection Qdrant et réinitialisation du fichier de suivi).

## 4. Infrastructure Externe et Intégration LLM

Le proxy RAG ne contient pas de LLM lui-même, mais interagit avec un service externe via une API HTTP :

-   **Serveur LLM** : Le modèle de langage utilisé est `Qwen3-Coder`.
-   **Fournisseur de Service** : Ce modèle est servi par une instance d'**Ollama**.
-   **Format d'API** : L'instance Ollama est configurée pour exposer une **API compatible avec celle d'OpenAI**.
-   **Reverse Proxy et Sécurité** : Un serveur web **Apache** est placé en tant que reverse proxy devant l'instance Ollama pour gérer l'authentification par clé d'API.

## 5. Fonctionnalités Clés

### Gestion Robuste des Erreurs
- Type `AppError` personnalisé (basé sur `thiserror`) centralisant tous les types d'erreurs
- Élimination complète des paniques (`unwrap`, `expect`) au profit d'une propagation propre des erreurs
- Implémentation de `IntoResponse` pour convertir automatiquement les erreurs en réponses HTTP appropriées
- Le serveur ne crashe pas en cas d'imprévu et retourne des codes d'erreur HTTP appropriés

### Logging Structuré
- Utilisation de `tracing` pour un logging professionnel avec niveaux de sévérité (info, warn, error)
- Remplacement complet des `println!` et `eprintln!` par des macros de logging
- Timestamps et contexte pour faciliter le débogage

### Architecture Modulaire
- **Clients API Centralisés** : `OllamaClient` et `LlmClient` encapsulent les appels HTTP pour éviter la duplication
- **Chargement Trait-based** : Trait `DocumentLoader` avec implémentations spécifiques facilitant l'ajout de nouveaux formats
- **Injection de Dépendances** : Configuration chargée une fois et partagée via `State<Arc<Config>>`

### Compatibilité et Robustesse
- Mode `--passthrough` pour le débogage sans traitement RAG
- Préservation de la structure JSON originale des requêtes pour compatibilité maximale avec les clients
- Gestion robuste des fichiers PDF problématiques via `catch_unwind`
- Support des formats texte, PDF et DOCX

## 6. Gestion des Erreurs

### Type `AppError`

Un type d'erreur personnalisé `AppError` a été défini dans `src/lib.rs` en utilisant la crate `thiserror`. Ce type énumère toutes les catégories d'erreurs possibles dans l'application :

- `Io`: Erreurs d'entrée/sortie (fichiers, etc.)
- `Toml`: Erreurs de parsing de configuration
- `Reqwest`: Erreurs réseau et HTTP
- `Json`: Erreurs de sérialisation/désérialisation JSON
- `Qdrant`: Erreurs spécifiques à l'API Qdrant
- `Config`: Erreurs de validation de configuration
- `Pdf`: Erreurs lors de l'extraction de texte PDF
- `Docx`: Erreurs lors de l'extraction de texte DOCX
- `Llm`: Erreurs lors de la communication avec le LLM
- `Unknown`: Erreurs génériques ou non classifiées

### Intégration avec Axum

Pour le serveur proxy RAG (`rag_proxy`), `AppError` implémente le trait `IntoResponse` d'Axum. Cela permet de convertir automatiquement les erreurs propagées depuis les handlers en réponses HTTP appropriées avec les codes de statut corrects (500 Internal Server Error, 502 Bad Gateway, 400 Bad Request, etc.) et un corps JSON structuré décrivant l'erreur.

### Avantages

1.  **Stabilité** : Le serveur ne crashe pas en cas d'erreur inattendue.
2.  **Clarté** : Les signatures de fonction `Result<T, AppError>` indiquent clairement les possibilités d'échec.
3.  **Débogage** : Les erreurs sont typées et contiennent des messages contextuels, facilitant le diagnostic.
4.  **Expérience Client** : Les clients reçoivent des réponses d'erreur HTTP standards et informatives.
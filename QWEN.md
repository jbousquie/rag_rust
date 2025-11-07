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

L'objectif est de fournir une solution robuste qui peut être facilement intégrée dans des architectures existantes, en tirant parti de la sécurité et des performances de Rust.

## 2. Architecture Générale

Le système est divisé en deux processus distincts :

-   **Le Binaire d'Indexation (`index_documents`)** : Un outil en ligne de commande qui parcourt le répertoire `data_sources/`, traite les fichiers qu'il contient et peuple la base de données Qdrant. Ce processus est destiné à être exécuté manuellement ou de manière planifiée lorsque les sources de données sont mises à jour.
-   **Le Serveur Proxy RAG (`rag_proxy`)** : Un service HTTP long-running qui expose un point d'accès (endpoint) pour les requêtes des utilisateurs.

Le flux d'une requête utilisateur est le suivant :
1.  Un client envoie une requête HTTP (contenant une question) au serveur `rag_proxy`.
2.  Le serveur transforme la question en un vecteur d'embedding.
3.  Il utilise ce vecteur pour interroger la base de données Qdrant et récupérer les fragments de documents les plus similaires (le contexte).
4.  Il construit une invite (prompt) enrichie, combinant la question originale de l'utilisateur et le contexte récupéré.
5.  Il envoie cette invite au service LLM distant.
6.  La réponse du LLM est renvoyée au client.

## 3. Organisation du Code

-   **Configuration Centrale** : Toute la configuration du projet est centralisée dans le fichier `config.toml`. Ce fichier contient tous les paramètres nécessaires : chemins des sources de données, paramètres du proxy RAG (port et host), configuration de l'API LLM (endpoint, modèle, clé d'API), et configuration de Qdrant (host, port, clé d'API). Cette approche centralisée simplifie la gestion des paramètres et permet de facilement personnaliser le comportement du système sans modification de code.

Le projet est structuré en modules clairs pour séparer les responsabilités :

-   `Cargo.toml` : Définit les métadonnées du projet et ses dépendances (axum, reqwest, qdrant-client, tokio, etc.).
-   `src/main.rs` : Point d'entrée de l'application. Il est responsable de parser les arguments de la ligne de commande pour lancer soit le processus d'indexation, soit le serveur proxy.
-   `src/lib.rs` : Contient du code partagé et des utilitaires qui peuvent être utilisés par les deux binaires (par exemple, la configuration, la gestion des erreurs).

### `src/common/`
-   `mod.rs` : Définit les types de données, les structures d'erreur personnalisées (`Error`), les constantes et toute autre logique partagée à travers les modules `indexing` et `rag_proxy`.

### `src/indexing/`
Ce module gère tout le processus de transformation des documents bruts en vecteurs stockés.
-   `mod.rs` : Déclare les sous-modules et expose la fonction principale d'orchestration de l'indexation.
-   `loader.rs` : Fonctions pour charger le contenu de différents types de fichiers (ex: `.txt`, `.pdf`, `.docx`) depuis le répertoire `data_sources/`.
-   `chunker.rs` : Logique pour découper les textes chargés en fragments (chunks) de taille gérable, en utilisant des stratégies pour préserver le sens sémantique.
-   `indexer.rs` : Orchestre la génération des embeddings pour chaque fragment en appelant le LLM et stocke ensuite les paires (fragment, vecteur) dans la collection Qdrant.
-   `file_tracker.rs` : Gère le suivi des fichiers indexés pour éviter de re-indexer les fichiers non modifiés.

### `src/rag_proxy/`
Ce module contient toute la logique du serveur HTTP.
-   `mod.rs` : Déclare les sous-modules et expose la fonction principale pour démarrer le serveur.
-   `server.rs` : Configure et lance le serveur web `axum`, définit les routes et attache les gestionnaires (handlers).
-   `handler.rs` : Contient la logique principale de traitement d'une requête HTTP. C'est ici que le flux RAG (embedding, recherche, appel LLM) est exécuté.
-   `retriever.rs` : Gère spécifiquement l'interaction avec Qdrant. Il prend une question, la vectorise et effectue la recherche de similarité pour récupérer le contexte.
-   `llm_caller.rs` : Gère la communication avec le LLM distant. Il est responsable de la construction du prompt final et de l'envoi de la requête HTTP à l'API du LLM.

### `data_sources/`
-   Ce répertoire contient les documents bruts qui serviront de base de connaissances pour le RAG. Il est ignoré par Git par défaut (sauf un fichier `.gitkeep`).

### `binaries/`
-   Ce répertoire est destiné à contenir les exécutables compilés (`index_documents` et `rag_proxy`). Il devrait être ajouté au `.gitignore`.

## 4. Infrastructure Externe et Intégration LLM

Le proxy RAG ne contient pas de LLM lui-même, mais interagit avec un service externe via une API HTTP. La configuration de cette infrastructure est la suivante :

-   **Serveur LLM** : Le modèle de langage utilisé est `Qwen3-Coder30b`.
-   **Fournisseur de Service** : Ce modèle est servi par une instance d'**Ollama**.
-   **Format d'API** : L'instance Ollama est configurée pour exposer une **API compatible avec celle d'OpenAI**. Cela permet d'utiliser des clients HTTP standards ou des bibliothèques comme `openai-rs` pour communiquer avec le modèle.
-   **Reverse Proxy et Sécurité** : Un serveur web **Apache** est placé en tant que reverse proxy devant l'instance Ollama.
-   **Rôle d'Apache** :
    1.  **Terminaison de la connexion** : Il peut gérer le HTTPS, bien que l'instance Ollama soit exposée en HTTP en interne.
    2.  **Authentification** : Apache est chargé de **gérer l'authentification par clé d'API**. Toute requête entrante doit contenir une clé d'API valide (par exemple, dans un en-tête `Authorization: Bearer VOTRE_CLÉ`). Apache valide cette clé avant de transmettre la requête à Ollama.

Le module `llm_caller.rs` dans le code Rust doit donc être implémenté pour inclure cette clé d'API dans chaque requête envoyée au LLM. La clé elle-même doit être gérée de manière sécurisée via des variables d'environnement ou un système de gestion de secrets.

## 5. Changements Réalisés pour la Compilation

Les modifications suivantes ont été apportées pour permettre la compilation du projet :

1. **Correction des fonctions main asynchrones** : Les fonctions `main` dans `src/indexing/main.rs` et `src/rag_proxy/main.rs` ont été converties de `async` à `sync` pour éviter les erreurs de compilation liées à l'utilisation incorrecte de `async` dans les binaires.
2. **Correction des imports de modules** : Les imports dans `src/indexing/main.rs` ont été mis à jour pour utiliser le bon chemin de module (`rag_rust::common::Config` au lieu de `crate::common::Config`).
3. **Création du module commun** : Le module `src/common/mod.rs` a été créé pour centraliser la structure `Config` et les types partagés.
4. **Suppression du fichier main.rs redondant** : Le fichier `src/main.rs` a été supprimé car il causait des conflits de module avec les binaires.
5. **Mise à jour de la documentation** : Le README.md a été mis à jour pour refléter les changements apportés.
6. **Implémentation de la gestion des fichiers de suivi** : Le fichier de suivi des documents indexés peut maintenant être configuré via `config.toml` dans la section `[indexing]` avec la clé `file_tracker_path`.
7. **Correction des erreurs de compilation** : Correction des problèmes d'import, de dépendances et d'implémentation des modules d'indexation pour permettre la compilation réussie du projet.
6. **Implémentation de la gestion des fichiers de suivi** : Le fichier de suivi des documents indexés peut maintenant être configuré via `config.toml` dans la section `[indexing]` avec la clé `file_tracker_path`.

# SPOOLER AGENT SGI

Es un servicio nativo de segundo plano (daemon) diseñado para ejecutarse localmente en la red de la estación de impresión. Actúa como un puente asíncrono y desacoplado entre el hardware físico (impresoras) y la API del servicio en la nube, garantizando la resiliencia operativa del negocio y la integridad de los datos de facturación.

# *DOCUMENTACION*

## *DIAGRAMA ENTIDAD - RELACION*

```mermaid

erDiagram
    role ||--o{ user : "assigned_to"
    user ||--o{ audit_log : "generates"
    user ||--o{ printer_log : "manages"
    user ||--o{ cash_closing : "opens_or_closes"
    user ||--o{ order : "registers"
    cash_closing ||--o{ order : "contains"
    order ||--o{ printer_log : "generates"
    order ||--o{ transaction : "has"
    order ||--|{ order_details : "breaks_down"
    transaction ||--o{ payment_list : "contains"
    payment_method ||--o{ payment_list : "defines"
    product ||--o{ order_details : "included_in"
    service ||--o{ order_details : "included_in"
    category ||--o{ product : "classifies"
    service ||--|{ service_details : "defines"
    product ||--o{ service_details : "consumes"

    role {
        int ID PK
        varchar name
        varchar description
        int level
    }

    user {
        int ID PK
        varchar username
        varchar password
        varchar name
        varchar lastname
        varchar email
        int role_ID FK
    }

    audit_log {
        int ID PK
        timestamp date
        varchar event_type
        varchar descripción
        int user_ID FK
    }

    cash_closing {
        int ID PK
        timestamp date_open
        timestamp date_close
        numeric cash_total
        numeric digital_total
        int paper_total
        int user_opening_ID FK
        int user_closing_ID FK
    }

    printer_log {
        int ID PK
        int pages
        varchar printer_name
        varchar printer_status
        text error_log
        int order_ID FK
        int user_ID FK
    }

    order {
        int ID PK
        timestamp date
        varchar description
        varchar status
        int user_ID FK
        int cash_closing_ID FK
    }

    transaction {
        int ID PK
        varchar status
        numeric total
        timestamp date
        int order_ID FK
    }

    payment_method {
        int ID PK
        varchar name
        varchar description
    }

    payment_list {
        int ID PK
        varchar reference
        numeric amount
        int payment_method_ID FK
        int transaction_ID FK
    }

    order_details {
        int ID PK
        int faces
        numeric subtotal
        int quantity
        int order_ID FK
        int service_ID FK
        int product_ID FK
    }

    product {
        int ID PK
        varchar name
        varchar description
        int stock_quantity
        numeric unit_cost
        int reorder_point
        int category_ID FK
    }

    category {
        int ID PK
        varchar name
        varchar description
        varchar type
    }

    service {
        int ID PK
        varchar name
        varchar description
        numeric price
        boolean is_special
    }

    service_details {
        int ID PK
        int quantity_consumed
        int service_ID FK
        int product_ID FK
    }

```

## *DIAGRAMAS DE FLUJO*

### *Proceso de Análisis Binario de Archivos Spool y Conciliación Homóloga de Páginas*

```mermaid

flowchart TD
    Start([Inicio: Trabajo recibido de la cola local]) --> Step1[Paso 1: Abrir flujo binario de archivo temporal]
    Step1 --> ReadBytes[Leer siguiente bloque de bytes]
    
    ReadBytes --> IsEOF{¿Es Fin del Archivo EOF?}
    
    %% Ciclo de Análisis de archivo (PCL/PostScript)
    IsEOF -- No --> ParseControl[Analizar comandos de control PCL / PostScript]
    ParseControl --> IsPageMarker{¿Se detectó marcador de página?}
    IsPageMarker -- No --> ReadBytes
    IsPageMarker -- Si --> IncTheoretical[Incrementar contador: Paginas_Teoricas ++]
    IncTheoretical --> ReadBytes
    
    %% Lanzamiento al Spooler del SO
    IsEOF -- Si --> Step2[Paso 2: Registrar y lanzar trabajo en OS Spooler]
    Step2 --> SetPrinting[Cambiar FSM a estado: PRINTING]
    SetPrinting --> Step3[Paso 3: Consultar estado del Job vía Win32/CUPS]
    
    %% Máquina de Estados del Hardware
    Step3 --> HWStatus{¿Cuál es el estado actual del hardware?}
    
    HWStatus -- "Error / Atasco / Sin Papel" --> SetError[Cambiar FSM a estado: ERROR]
    SetError --> CaptureError[Capturar código de error y congelar contadores]
    CaptureError --> Step4
    
    HWStatus -- "Activo / Imprimiendo" --> DriverReport{¿El driver reporta página expulsada?}
    DriverReport -- No --> Step3
    DriverReport -- Si --> IncPhysical[Incrementar contador: Paginas_Fisicas ++]
    IncPhysical --> Step3
    
    HWStatus -- "Completado" --> SetCompleted[Cambiar FSM a estado: COMPLETED]
    
    %% Fase de Auditoría y Cierre
    SetCompleted --> Step4[Paso 4: Evaluar Auditoría <br> ¿Paginas_Teoricas == Paginas_Fisicas?]
    
    Step4 -- No --> LogDiscrepancy[Generar log: <br> status: 'warning', error: 'discrepancia']
    Step4 -- Si --> LogSuccess[Generar log: <br> status: 'success', error: 'none']
    
    LogDiscrepancy --> Step5[Paso 5: Encolar payload para sincronización con la nube]
    LogSuccess --> Step5
    
    Step5 --> End([Fin de Algoritmo])

```
### *Algoritmo de Intercepción de Spool Local, Monitoreo por FSM y Conciliación Física de Páginas*

```mermaid

%%{init: {'theme': 'base', 'themeVariables': { 'primaryColor': '#ffffff', 'primaryBorderColor': '#777', 'primaryTextColor': '#333', 'lineColor': '#777', 'secondaryColor': '#f4f4f4', 'tertiaryColor': '#ffffff'}}%%}

flowchart TD
    %% Node Styles
    classDef startend fill:#dae8fc,stroke:#6c8ebf,stroke-width:1px,color:#000000;
    classDef process fill:#d5e8d4,stroke:#82b366,stroke-width:1px,color:#000000;
    classDef decision fill:#fff2cc,stroke:#d6b656,stroke-width:1px,color:#000000;
    classDef auditCheck fill:#fff2cc,stroke:#d6b656,stroke-width:1px,color:#000000;

    %% START NODE
    A([Inicio: Trabajo recibido de la cola local]):::startend
    A --> B

    %% PART 1: BINARY FILE ANALYSIS (PCL/POSTSCRIPT)
    B[Paso 1: Abrir flujo binario de archivo temporal]:::process
    B --> C
    C[Leer siguiente bloque de bytes]:::process
    C --> D{¿Es Fin de Archivo EOF?}:::decision

    %% Branch NO (from EOF check)
    D -- No --> E[Analizar comandos de control PCL / PostScript]:::process
    E --> F{¿Se detectó marcador de página?}:::decision

    %% Branch NO (from Page Marker check) loops back
    F -- No --> C

    %% Branch YES (from Page Marker check)
    F -- Sí --> G[Incrementar contador: Paginas_Teoricas ++]:::process
    %% loops back from YES
    G --> C

    %% Branch YES (from EOF check)
    D -- Sí --> H[Paso 2: Registrar y lanzar trabajo en OS Spooler]:::process
    H --> I[Cambiar FSM a estado: PRINTING]:::process
    I --> J[Paso 3: Consultar estado del Job vía Win32/CUPS]:::process
    J --> K{¿Cuál es el estado actual del hardware?}:::decision

    %% PART 2: HARDWARE FSM & PHYSICAL PAGE CHECK
    %% Branch ACTIVE (from Hardware check)
    K -- "Activo / Imprimiendo" --> L{¿El driver reporta página expulsada?}:::decision

    %% Branch NO (from Page Expelled check) loops back
    L -- No --> J

    %% Branch YES (from Page Expelled check)
    L -- Sí --> M[Incrementar contador: Paginas_Fisicas ++]:::process
    %% loops back to step 3
    M --> J

    %% Branch COMPLETED (from Hardware check)
    K -- Completado --> N[Cambiar FSM a estado: COMPLETED]:::process

    %% Branch ERROR (from Hardware check)
    K -- "Error / Atasco / Sin Papel" --> O[Cambiar FSM a estado: ERROR]:::process
    O --> P[Capturar código de error y congelar contadores]:::process

    %% Merging of branches to the next phase
    N --> Q
    P --> Q

    %% PART 3: AUDIT AND COMPLETION
    Q([Paso 4: Evaluar Auditoría: ¿Paginas_Teoricas == Paginas_Fisicas?]):::auditCheck

    %% Decision branches out of standard box drawn in diagram (atypical mermaid, so using labels on arrows to box below)
    Q -->|No| R[Generar log: status: 'warning', error: 'discrepancia']:::process
    Q -->|Sí| S[Generar log: status: 'success', error: 'none']:::process

    %% Merging of audit branches
    R --> T
    S --> T

    %% Final Steps
    T[Paso 5: Encolar payload para sincronización con la nube]:::process
    T --> U

    %% END NODE
    U([Fin de Algoritmo]):::startend

```

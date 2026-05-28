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

flowchart TD
    Start([Inicio: Trabajo recibido de la cola local]) --> Step1[Paso 1: Abrir flujo binario de archivo temporal]
    Step1 --> ReadBytes[Leer siguiente bloque de bytes]
    
    ReadBytes --> IsEOF{¿Es Fin del Archivo EOF?}
    
    %% Ciclo de Análisis de archivo (PCL/PostScript)
    IsEOF -- No --> ParseControl[Analizar comandos de control PCL / PostScript]
    ParseControl --> IsPageMarker{¿Se detectó marcador de página?}
    IsPageMarker -- No --> ReadBytes
    IsPageMarker -- Sí --> IncTheoretical[Incrementar contador: Paginas_Teoricas ++]
    IncTheoretical --> ReadBytes
    
    %% Lanzamiento al Spooler del SO
    IsEOF -- Sí --> Step2[Paso 2: Registrar y lanzar trabajo en OS Spooler]
    Step2 --> SetPrinting[Cambiar FSM a estado: PRINTING]
    SetPrinting --> Step3[Paso 3: Consultar estado del Job vía Win32/CUPS]
    
    %% Máquina de Estados del Hardware
    Step3 --> HWStatus{¿Cuál es el estado actual del hardware?}
    
    HWStatus -- "Error / Atasco / Sin Papel" --> SetError[Cambiar FSM a estado: ERROR]
    SetError --> CaptureError[Capturar código de error y congelar contadores]
    CaptureError --> Step4
    
    HWStatus -- "Activo / Imprimiendo" --> DriverReport{¿El driver reporta página expulsada?}
    DriverReport -- No --> Step3
    DriverReport -- Sí --> IncPhysical[Incrementar contador: Paginas_Fisicas ++]
    IncPhysical --> Step3
    
    HWStatus -- "Completado" --> SetCompleted[Cambiar FSM a estado: COMPLETED]
    
    %% Fase de Auditoría y Cierre
    SetCompleted --> Step4[Paso 4: Evaluar Auditoría <br> ¿Paginas_Teoricas == Paginas_Fisicas?]
    
    Step4 -- No --> LogDiscrepancy[Generar log: <br> status: 'warning', error: 'discrepancia']
    Step4 -- Sí --> LogSuccess[Generar log: <br> status: 'success', error: 'none']
    
    LogDiscrepancy --> Step5[Paso 5: Encolar payload para sincronización con la nube]
    LogSuccess --> Step5
    
    Step5 --> End([Fin de Algoritmo])

    %% Estilos para bloques de error/advertencia
    style SetError fill:#E80909,stroke:#9c0006,stroke-width:1px;
    style LogDiscrepancy fill:#E80909,stroke:#9c0006,stroke-width:1px;

```

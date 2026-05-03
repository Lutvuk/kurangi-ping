# API Contract Template (OpenAPI 3.0+)

**File Name Format:** `docs/api/[service-name].yaml` or `docs/openapi.yaml`

```yaml
openapi: 3.0.3
info:
  title: [API Name]
  description: [Comprehensive API description]
  version: 1.0.0
  contact:
    name: API Team

servers:
  - url: https://api.example.com/v1
    description: Production
  - url: http://localhost:3000/v1
    description: Development

tags:
  - name: [Resource Name]
    description: Operations related to [Resource]

paths:
  # Example: 1:N Resource
  /[resources]:
    get:
      summary: List [resources]
      operationId: list[Resources]
      tags:
        - [Resource Name]
      parameters:
        - $ref: '#/components/parameters/Page'
        - $ref: '#/components/parameters/Limit'
      responses:
        '200':
          description: Successful response
          content:
            application/json:
              schema:
                type: object
                properties:
                  data:
                    type: array
                    items:
                      $ref: '#/components/schemas/[Resource]Simple'
                  meta:
                    $ref: '#/components/schemas/PaginationMeta'
    post:
      summary: Create a new [resource]
      operationId: create[Resource]
      tags:
        - [Resource Name]
      requestBody:
        required: true
        content:
          application/json:
            schema:
              $ref: '#/components/schemas/[Resource]Create'
      responses:
        '201':
          description: Created successfully
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/[Resource]'

  /[resources]/{id}:
    parameters:
      - $ref: '#/components/parameters/ResourceId'
    get:
      summary: Get [resource] by ID
      operationId: get[Resource]
      tags:
        - [Resource Name]
      responses:
        '200':
          description: Successful response
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/[Resource]'
        '404':
          $ref: '#/components/responses/NotFound'
    put:
      summary: Update [resource] fully
      operationId: update[Resource]
      tags:
        - [Resource Name]
      requestBody:
        required: true
        content:
          application/json:
            schema:
              $ref: '#/components/schemas/[Resource]Update'
      responses:
        '200':
          description: Updated successfully
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/[Resource]'

components:
  schemas:
    # Standard Error Response
    Error:
      type: object
      required:
        - code
        - message
      properties:
        code:
          type: string
          example: AUTH_001
        message:
          type: string
          example: Unauthorized access
        details:
          type: array
          items:
            type: object
            properties:
              field:
                type: string
              issue:
                type: string

    # Pagination Meta
    PaginationMeta:
      type: object
      properties:
        total:
          type: integer
        page:
          type: integer
        limit:
          type: integer
        pages:
          type: integer

    # Example Resource Schemas
    [Resource]:
      type: object
      required:
        - id
        - name
      properties:
        id:
          type: string
          format: uuid
        name:
          type: string
        created_at:
          type: string
          format: date-time

    [Resource]Simple:
      type: object
      properties:
        id:
          type: string
          format: uuid
        name:
          type: string

    [Resource]Create:
      type: object
      required:
        - name
      properties:
        name:
          type: string

  parameters:
    Page:
      name: page
      in: query
      schema:
        type: integer
        default: 1
    Limit:
      name: limit
      in: query
      schema:
        type: integer
        default: 20
    ResourceId:
      name: id
      in: path
      required: true
      schema:
        type: string
        format: uuid

  responses:
    NotFound:
      description: Resource not found
      content:
        application/json:
          schema:
            $ref: '#/components/schemas/Error'
    Unauthorized:
      description: Authentication required
      content:
        application/json:
          schema:
            $ref: '#/components/schemas/Error'

  securitySchemes:
    BearerAuth:
      type: http
      scheme: bearer
      bearerFormat: JWT

security:
  - BearerAuth: []
```

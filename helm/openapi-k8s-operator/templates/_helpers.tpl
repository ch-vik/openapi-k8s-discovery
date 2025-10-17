{{/*
Expand the name of the chart.
*/}}
{{- define "openapi-k8s-operator.name" -}}
{{- default .Chart.Name .Values.nameOverride | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Create a default fully qualified app name.
We truncate at 63 chars because some Kubernetes name fields are limited to this (by the DNS naming spec).
If release name contains chart name it will be used as a full name.
*/}}
{{- define "openapi-k8s-operator.fullname" -}}
{{- if .Values.nameOverride }}
{{- printf "%s-%s" .Release.Name .Values.nameOverride | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- printf "%s-%s" .Release.Name .Chart.Name | trunc 63 | trimSuffix "-" }}
{{- end }}
{{- end }}

{{/*
Create chart name and version as used by the chart label.
*/}}
{{- define "openapi-k8s-operator.chart" -}}
{{- printf "%s-%s" .Chart.Name .Chart.Version | replace "+" "_" | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Common labels
*/}}
{{- define "openapi-k8s-operator.labels" -}}
helm.sh/chart: {{ include "openapi-k8s-operator.chart" . }}
{{ include "openapi-k8s-operator.selectorLabels" . }}
{{- if .Chart.AppVersion }}
app.kubernetes.io/version: {{ .Chart.AppVersion | quote }}
{{- end }}
app.kubernetes.io/managed-by: {{ .Release.Service }}
{{- with .Values.commonLabels }}
{{ toYaml . }}
{{- end }}
{{- end }}

{{/*
Selector labels
*/}}
{{- define "openapi-k8s-operator.selectorLabels" -}}
app.kubernetes.io/name: {{ include "openapi-k8s-operator.name" . }}
app.kubernetes.io/instance: {{ .Release.Name }}
{{- end }}

{{/*
Create the name of the service account to use
*/}}
{{- define "openapi-k8s-operator.serviceAccountName" -}}
{{- if .Values.operator.serviceAccount.create }}
{{- default (include "openapi-k8s-operator.fullname" .) .Values.operator.serviceAccount.name }}
{{- else }}
{{- default "default" .Values.operator.serviceAccount.name }}
{{- end }}
{{- end }}

{{/*
Create the name of the operator deployment/statefulset
*/}}
{{- define "openapi-k8s-operator.operatorName" -}}
{{- if .Values.operator.deployment.name }}
{{- .Values.operator.deployment.name }}
{{- else }}
{{- include "openapi-k8s-operator.fullname" . }}
{{- end }}
{{- end }}

{{/*
Create the name of the openapi server deployment
*/}}
{{- define "openapi-k8s-operator.serverName" -}}
{{- if .Values.openapiServer.deployment.name }}
{{- .Values.openapiServer.deployment.name }}
{{- else }}
{{- printf "%s-server" (include "openapi-k8s-operator.fullname" .) }}
{{- end }}
{{- end }}

{{/*
Create the name of the openapi server service
*/}}
{{- define "openapi-k8s-operator.serverServiceName" -}}
{{- if .Values.openapiServer.service.name }}
{{- .Values.openapiServer.service.name }}
{{- else }}
{{- printf "%s-server" (include "openapi-k8s-operator.fullname" .) }}
{{- end }}
{{- end }}

{{/*
Validate namespace configuration
*/}}
{{- define "openapi-k8s-operator.validateNamespaces" -}}
{{- if and (ne .Values.operator.config.watchNamespaces "") (ne .Values.operator.config.watchNamespaces "all") (not (contains "," .Values.operator.config.watchNamespaces)) }}
{{- fail (printf "Invalid watchNamespaces configuration: '%s'. Must be empty string (current namespace), 'all' (all namespaces), or comma-separated list of namespaces" .Values.operator.config.watchNamespaces) }}
{{- end }}
{{- end }}

{{/*
Determine if cluster-wide RBAC is needed
*/}}
{{- define "openapi-k8s-operator.clusterWideRBAC" -}}
{{- include "openapi-k8s-operator.validateNamespaces" . }}
{{- if or (eq .Values.operator.config.watchNamespaces "all") (contains "," .Values.operator.config.watchNamespaces) }}
{{- true }}
{{- else }}
{{- .Values.operator.rbac.clusterWide }}
{{- end }}
{{- end }}

{{/*
Determine if cluster-wide network policy is needed
*/}}
{{- define "openapi-k8s-operator.clusterWideNetworkPolicy" -}}
{{- include "openapi-k8s-operator.validateNamespaces" . }}
{{- if or (eq .Values.operator.config.watchNamespaces "all") (contains "," .Values.operator.config.watchNamespaces) }}
{{- true }}
{{- else }}
{{- .Values.operator.networkPolicy.allowClusterWide }}
{{- end }}
{{- end }}

{{/*
Create the image name
*/}}
{{- define "openapi-k8s-operator.image" -}}
{{- $registry := .Values.global.imageRegistry | default "" }}
{{- $repository := .Values.operator.image.repository }}
{{- $tag := .Values.operator.image.tag | default .Chart.AppVersion }}
{{- if $registry }}
{{- printf "%s/%s:%s" $registry $repository $tag }}
{{- else }}
{{- printf "%s:%s" $repository $tag }}
{{- end }}
{{- end }}

{{/*
Create the openapi server image name
*/}}
{{- define "openapi-k8s-operator.serverImage" -}}
{{- $registry := .Values.global.imageRegistry | default "" }}
{{- $repository := .Values.openapiServer.image.repository }}
{{- $tag := .Values.openapiServer.image.tag }}
{{- if $registry }}
{{- printf "%s/%s:%s" $registry $repository $tag }}
{{- else }}
{{- printf "%s:%s" $repository $tag }}
{{- end }}
{{- end }}

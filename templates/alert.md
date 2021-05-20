{% if notification.status == "firing" %}
### <font color="#FF0000">告警信息</font>
{% for alert in notification.alerts %}
**告警名称**: {{ alert.labels.alertname }}

**告警级别**: {{ alert.labels.severity }}

**报警描述**: {{ alert.annotations.description }}

**开始时间**: {{ alert.startsAt | date(timezone="Asia/Shanghai",format="%Y-%m-%d %H:%M:%S") }}

{% endfor %}
{% else %}
### <font color="#0000FF">恢复信息</font>
{% for alert in notification.alerts %}

**告警名称**: {{ alert.labels.alertname }}

**告警级别**: {{ alert.labels.severity }}

**报警描述**: {{ alert.annotations.description }}

**开始时间**: {{ alert.startsAt | date(timezone="Asia/Shanghai",format="%Y-%m-%d %H:%M:%S") }}

**恢复时间**: {{ alert.endsAt | date(timezone="Asia/Shanghai",format="%Y-%m-%d %H:%M:%S") }}

{% endfor %}
{% endif %}
